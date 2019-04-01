use crate::change::OSMObjectChange;
use crate::change_iterator::OSMObjectChangeIterator;
use crate::object::{OSMObject, OSMObjectSpecifics, OSMObjectType};
use crate::utils;
use crate::Result;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use reqwest;
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{self, Deserializer};
use sqlitemap::SqliteMap;
use std::cell::RefCell;
use std::fs;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::iter;
use std::time::Instant;
use tempfile::tempfile;

fn translate_type_shortcut(shortcut: char) -> &'static str {
    match shortcut {
        'n' => "node",
        'w' => "way",
        'r' => "relation",
        _ => unreachable!(),
    }
}

fn format_query(timeout: u32, query: &str) -> String {
    format!(
        "[out:json][timeout:{timeout}];{query};out meta;",
        timeout = timeout,
        query = query
    )
}

fn format_data_retrieval(area: &str) -> String {
    format!(r#"((area["name"="{area}"];node(area);area["name"="{area}"];way(area);area["name"="{area}"];rel(area);>>;);>>;)"#, area = area)
}

pub struct OSMObjectManager {
    current_api_url_idx: RefCell<usize>,
    api_urls: Vec<&'static str>,
    cache_conn: Option<Connection>,
    http_client: reqwest::Client,
}

impl OSMObjectManager {
    pub fn new() -> Self {
        let conn = Connection::open("entity_cache.db").expect("Could not create connection.");
        conn.execute("PRAGMA SYNCHRONOUS=off", &[]).unwrap();
        conn.execute("BEGIN", &[]).unwrap();
        let client = reqwest::Client::builder()
            .timeout(None)
            .build()
            .expect("Failed to create http client.");
        OSMObjectManager {
            api_urls: vec![
                "https://z.overpass-api.de/api",
                "https://lz4.overpass-api.de/api",
            ],
            current_api_url_idx: RefCell::new(0 as usize),
            cache_conn: Some(conn),
            http_client: client,
        }
    }

    pub fn get_cache(&self) -> SqliteMap<'_> {
        SqliteMap::new(
            &self.cache_conn.as_ref().unwrap(),
            "raw_entities",
            "text",
            "text",
        )
        .unwrap()
    }

    fn cache_object_into(&self, cache: &mut SqliteMap<'_>, object: &OSMObject) {
        let serialized =
            serde_json::to_string(&object).expect("Could not serialize object for caching.");
        cache
            .insert::<String>(&object.unique_id(), &serialized)
            .expect("Could not cache object.");
    }

    fn has_object(&self, id: &str) -> bool {
        let mut cache = self.get_cache();
        cache.contains_key(&id).expect("Cache query failed.")
    }

    fn get_cached_object(&self, id: &str) -> Option<OSMObject> {
        let mut cache = self.get_cache();
        cache
            .get::<String>(&id)
            .expect("Cache retrieval failed.")
            .map(|o| serde_json::from_str(&o).expect("Failed to deserialize cached object."))
    }

    fn next_api_url(&self) -> &'static str {
        if *self.current_api_url_idx.borrow() == self.api_urls.len() - 1{
            *self.current_api_url_idx.borrow_mut() = 0 as usize;
        }
        else {
            *self.current_api_url_idx.borrow_mut() += 1 as usize;
        }
        self.api_urls[*self.current_api_url_idx.borrow()]
    }

    fn run_query(&self, query: &str, result_to_tempfile: bool) -> Result<Box<dyn Read>> {
        let start = Instant::now();
        let url = self.next_api_url();
        let final_url = format!("{}/interpreter?data={}", url, query);
        debug!("Requesting resource {}", final_url);
        let mut resp = self.http_client.get(&final_url).send()?;
        match resp.status().as_u16() {
            429 => {
                warn!("Multiple requests, killing them and going to a different api host.");
                self.http_client
                    .get(&format!("{0}/kill_my_queries", &url))
                    .send()?;
                self.run_query(&query, result_to_tempfile)
            }
            200 => {
                debug!("Request successfully finished after {:?}.", start.elapsed());
                if !result_to_tempfile {
                Ok(Box::new(resp))
                }else {
                    let mut file = tempfile()?;
                    io::copy(&mut resp, &mut file)?;
                    file.seek(SeekFrom::Start(0))?;
                    Ok(Box::new(file))

                }
            }
            _ => {
                warn!("Unexpected status code {} from the server.", resp.status());
                self.run_query(&query, result_to_tempfile)
            }
        }
    }

    fn cache_objects_from(
        &self,
        readable: Box<dyn Read>,
        return_objects: bool,
    ) -> Result<Vec<OSMObject>> {
        let start = Instant::now();
        let mut cache = self.get_cache();
        let mut objects = Vec::new();
        let mut cached_readable = BufReader::with_capacity(65536, readable);
        let mut buf = [0; 1];
        loop {
            let _bytes = cached_readable.read(&mut buf)?;
            if String::from_utf8_lossy(&buf[..]) == "[" {
                break;
            }
        }
        // We're past the data header, only objects follow.
        loop {
            {
                let mut de = Deserializer::from_reader(&mut cached_readable);
                match OSMObject::deserialize(&mut de) {
                    Ok(obj) => {
                        self.cache_object_into(&mut cache, &obj);
                        if return_objects {
                            objects.push(obj);
                        }
                    }
                    Err(_e) => break,
                }
            }
            cached_readable.read_exact(&mut buf)?; // Skip a comma
        }
        debug!("Caching finished after {:?}", start.elapsed());
        self.commit_cache();
        Ok(objects)
    }

    pub fn lookup_objects_in(&self, area: &str) -> Result<()> {
        info!("Looking up all objects in area {}.", area);
        let query = format_query(900, &format_data_retrieval(area));
        let readable = self.run_query(&query, false)?;
        self.cache_objects_from(readable, false)?;
        Ok(())
    }

    fn lookup_objects(&self, ids: &mut [String]) -> Result<()> {
        const MAX_SIMULTANEOUSLY_QUERYED: usize = 1000000;
        let mut objects: Vec<OSMObject> = vec![];
        ids.sort_unstable_by_key(|oid| oid.chars().nth(0));
        for (entity_type, entity_ids) in &ids.iter().group_by(|oid| oid.chars().nth(0).unwrap()) {
            for mut chunk in &entity_ids.chunks(MAX_SIMULTANEOUSLY_QUERYED) {
                let ids_str = chunk.join(",");
                let query = format_query(
                    900,
                    &format!("{}(id:{})", translate_type_shortcut(entity_type), ids_str),
                );
                let readable = self.run_query(&query, false)?;
                objects.extend(self.cache_objects_from(readable, true)?);
            }
        }
        self.ensure_has_cached_dependencies_for(&objects)?;
        Ok(())
    }

    fn ensure_has_cached_dependencies_for(&self, objects: &[OSMObject]) -> Result<()> {
        use self::OSMObjectSpecifics::*;
        let mut missing = vec![];
        for object in objects {
            match object.specifics {
                Node { .. } => continue, // Nodes have no dependencies
                Way { ref nodes, .. } => {
                    for node in nodes {
                        let node_id = format!("n{}", node);
                        if !self.has_object(&node_id) {
                            debug!("Node with id {} missing.", node_id);
                            missing.push(node_id);
                        }
                    }
                }
                Relation { ref members, .. } => {
                    for member in members {
                        if !self.has_object(&member.unique_reference()) {
                            debug!("Object {} is missing.", member.unique_reference());
                            missing.push(member.unique_reference());
                        }
                    }
                }
            }
        }
        if !missing.is_empty() {
            info!(
                "Out of {} objects {} was missing.",
                objects.len(),
                missing.len()
            );
            self.lookup_objects(&mut missing[..])?;
        }
        Ok(())
    }

    fn related_ids_of<'a>(
        &'a self,
        object: &'a OSMObject,
    ) -> Result<Box<dyn Iterator<Item = (String, Option<String>)>>> {
        use self::OSMObjectSpecifics::*;
        match object.specifics {
            Node { .. } => Ok(Box::new(iter::empty())),
            Way { ref nodes, .. } => {
                self.ensure_has_cached_dependencies_for(&[object.clone()])?;
                Ok(Box::new(
                    nodes.clone().into_iter().map(|n| (format!("n{}", n), None)),
                ))
            }
            Relation { ref members, .. } => {
                self.ensure_has_cached_dependencies_for(&[object.clone()])?;
                Ok(Box::new(
                    members
                        .clone()
                        .into_iter()
                        .map(|m| (m.unique_reference(), Some(m.role))),
                ))
            }
        }
    }
    fn related_objects_of<'a>(
        &'a self,
        object: &'a OSMObject,
    ) -> Result<impl Iterator<Item = OSMObject> + 'a> {
        Ok(self.related_ids_of(&object)?.map(move |(id, maybe_role)| {
            let mut related = self.get_cached_object(&id).unwrap();
            OSMObjectManager::enrich_tags(&object, &mut related);
            if let Some(role) = maybe_role {
                related.tags.insert("role".to_string(), role);
            }
            related
        }))
    }

    pub fn get_object(&self, id: &str) -> Result<Option<OSMObject>> {
        if !self.has_object(&id) {
            self.lookup_objects(&mut [id.to_string()])?;
        }
        Ok(self.get_cached_object(&id))
    }

    fn enrich_tags(parent: &OSMObject, child: &mut OSMObject) {
        child
            .tags
            .insert("parent_id".to_string(), parent.unique_id());
    }

    fn get_way_coords(&self, way: &OSMObject) -> Result<Vec<(f64, f64)>> {
        use self::OSMObjectSpecifics::Node;
        let mut coords = vec![];
        for obj in self.related_objects_of(&way)? {
            match obj.specifics {
                Node { lon, lat } => {
                    coords.push((lon, lat));
                }
                _ => unreachable!(),
            }
        }
        Ok(coords)
    }

    pub fn get_geometry_as_wkt(&self, object: &OSMObject) -> Result<Option<String>> {
        use self::OSMObjectSpecifics::*;
        match object.specifics {
            Node { lon, lat } => Ok(Some(format!("POINT({} {})", lon, lat))),
            Way { .. } => {
                let mut coords = self.get_way_coords(&object)?;
                if coords.len() <= 1 {
                    warn!("One or zero nodes for object {}", object.unique_id());
                    return Ok(None);
                }
                if utils::object_should_have_closed_geometry(&object) && coords.len() > 2 {
                    utils::ensure_closed(&mut coords);
                    let coords_text = utils::coords_to_text(&coords);
                    Ok(Some(format!("POLYGON(({}))", coords_text)))
                } else {
                    let coords_text = utils::coords_to_text(&coords);
                    Ok(Some(format!("LINESTRING({})", coords_text)))
                }
            }
            Relation { .. } => {
                let empty = "".to_string();
                let geom_type = object.tags.get(&"type".to_string()).unwrap_or(&empty);
                if geom_type == &"multipolygon".to_string() {
                    let first_related = self.related_objects_of(&object)?.next().unwrap();
                    let multi;
                    match first_related
                        .tags
                        .get("role")
                        .unwrap_or(&"".to_string())
                        .as_ref()
                    {
                        "inner" | "outer" => {
                            multi = self.construct_multipolygon_from_complex_polygons(&object)?;
                        }
                        _ => {
                            multi = self.construct_multipolygon_from_polygons(&object)?;
                        }
                    }
                    if let Some(geom) = multi {
                        Ok(Some(geom))
                    } else {
                        self.create_geometry_collection(&object)
                    }
                } else {
                    self.create_geometry_collection(&object)
                }
            }
        }
    }

    fn create_geometry_collection(&self, object: &OSMObject) -> Result<Option<String>> {
        Ok(Some(format!(
            "GEOMETRYCOLLECTION({})",
            self.related_objects_of(&object)?
                .map(|o| self.get_geometry_as_wkt(&o))
                .filter_map(|g| g.ok())
                .filter_map(|g| g)
                .join(", ")
        )))
    }

    fn construct_multipolygon_from_polygons(&self, object: &OSMObject) -> Result<Option<String>> {
        let mut parts = vec![];
        for related in self.related_objects_of(&object)? {
            let rel_geom = self.get_geometry_as_wkt(&related)?.unwrap();
            if !rel_geom.starts_with("POLYGON") {
                warn!(
                    "Multipolygon promise broken for object with id {}",
                    related.unique_id()
                );
                return Ok(None);
            }
            parts.push(rel_geom.replace("POLYGON", ""));
        }
        Ok(Some(format!("MULTIPOLYGON({})", parts.join(", "))))
    }

    fn construct_multipolygon_from_complex_polygons(
        &self,
        object: &OSMObject,
    ) -> Result<Option<String>> {
        let mut inners = vec![];
        let mut outers = vec![];
        for related in self.related_objects_of(&object)? {
            if !related.tags.contains_key(&"role".to_string()) {
                warn!(
                    "Missing role specifier for object {} as part of geometry of object {}",
                    related.unique_id(),
                    object.unique_id()
                );
                return Ok(None);
            }
            if !(related.object_type() == OSMObjectType::Way) {
                warn!(
                    "Creation of a point sequence for object {} not supported/impossible.",
                    related.unique_id()
                );
                return Ok(None);
            }
            let points = self.get_way_coords(&related)?;
            match related.tags["role"].as_ref() {
                "inner" => inners.push(points),
                "outer" => outers.push(points),
                _ => {
                    warn!("Unknown multipolygon part tole {}.", related.tags["role"]);
                    return Ok(None);
                }
            }
        }
        utils::connect_polygon_segments(&mut inners);
        utils::connect_polygon_segments(&mut outers);
        if outers.len() != 1 && !inners.is_empty() {
            warn!("multiple outer ring(s) and some inner ring(s), geometry for object {} is ambiguous.", object.unique_id());
            return Ok(None);
        }
        let mut polys = vec![];
        // Because i could not manage to convince the loop that references are enough, the check must be done there
        let inners_is_empty = inners.is_empty();
        for inner in inners {
            if inner.len() < 4 {
                warn!("One of the inner polygons for object {} did not have enough points, falling back to a geometry collection.", object.unique_id());
                return Ok(None);
            }
            polys.push(format!(
                "(({}),({}))",
                utils::coords_to_text(&outers[0]),
                utils::coords_to_text(&inner)
            ));
        }
        if inners_is_empty {
            for outer in outers {
                if outer.len() < 4 {
                    warn!("One of the outer rings of object {} did not have enough points, falling back to a geometry collection.", object.unique_id());
                    return Ok(None);
                }
                polys.push(format!("(({}))", utils::coords_to_text(&outer)));
            }
        }
        if polys.len() == 1 {
            Ok(Some(format!("POLYGON{}", polys[0])))
        } else {
            Ok(Some(format!("MULTIPOLYGON({})", polys.join(", "))))
        }
    }

    fn commit_cache(&self) {
        self.cache_conn
            .as_ref()
            .unwrap()
            .execute("COMMIT", &[])
            .expect("Commit failed.");
    }
    pub fn lookup_differences_in(
        &self,
        area: &str,
        after: &DateTime<Utc>,
    ) -> Result<Box<Iterator<Item = Result<OSMObjectChange>>>> {
        let mut iterators = Vec::with_capacity(3);
        for kind in &["node", "way", "rel"] {
            let query = format!(
                "((area[\"name\"=\"{area}\"];{object_kind}(area);>>;);>>;)",
                area = area,
                object_kind = kind
            );
            let final_query = format!(
                "[out:xml][timeout:900][adiff:\"{after}\"];{query};out meta;",
                after = after.to_rfc3339(),
                query = query
            );
            info!("Looking up differences in area {}, {}s only.", area, kind);
            let readable = self.run_query(&final_query, true)?;
            iterators.push(OSMObjectChangeIterator::new(readable));
        }
        Ok(Box::new(iterators.into_iter().flat_map(|it| it)))
    }
}

impl Drop for OSMObjectManager {
    fn drop(&mut self) {
        let conn = self.cache_conn.take().unwrap();
        conn.close().expect("Failed to close cache connection.");
        fs::remove_file("entity_cache.db").expect("Could not remove cache.");
    }
}

pub fn cached_objects_in<'a>(
    cache: &'a mut SqliteMap<'_>,
) -> Box<(Iterator<Item = OSMObject>) + 'a> {
    Box::new(
        cache
            .iter::<String, String>()
            .unwrap()
            .filter_map(|pair| pair.ok())
            .map(|(_k, v)| serde_json::from_str(&v).unwrap()),
    )
}
