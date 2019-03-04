use crate::area::{Area, AreaState};
use crate::Result;
use crate::{area_messaging, diff_utils};
use crate::background_task_constants::*;
use crate::background_task::BackgroundTask;
use chrono::{DateTime, Utc};
use diesel::{Connection, SqliteConnection};
use osm_api::change::OSMObjectChangeType;
use osm_api::object_manager::OSMObjectManager;
use osm_db::area_db::AreaDatabase;
use osm_db::semantic_change::SemanticChange;
use osm_db::translation::translator;

fn update_area(area: &mut Area, conn: &SqliteConnection) -> Result<()> {
    info!("Updating area {}.", area.name);
    area.state = AreaState::GettingChanges;
    area.save(&conn)?;
    let after = if let Some(timestamp) = &area.newest_osm_object_timestamp {
        DateTime::parse_from_rfc3339(&timestamp)?.with_timezone(&Utc)
    } else {
        DateTime::from_utc(area.updated_at, Utc)
    };
    let manager = OSMObjectManager::new();
    let area_db = AreaDatabase::open_existing(&area.name)?;
    let mut first = true;
    for change in manager.lookup_differences_in(&area.name, &after)? {
        use OSMObjectChangeType::*;
        if first {
            area.state = AreaState::ApplyingChanges;
            area.save(&conn)?;
            first = false;
        }
        let change = change?;
        if change.new.is_some()
            && (area.newest_osm_object_timestamp.is_none()
                || change.new.as_ref().unwrap().timestamp
                    > *area.newest_osm_object_timestamp.as_ref().unwrap())
        {
            area.newest_osm_object_timestamp = Some(change.new.as_ref().unwrap().timestamp.clone());
        }
        let semantic_change = match change.change_type {
            Create => translator::translate(
                &change.new.expect("No new object for a create change"),
                &manager,
            )?
            .map(|o| {
                SemanticChange::creating(o.geometry, o.discriminator, o.data, o.effective_width)
            }),
            Delete => {
                let osm_id = change.old.expect("No old in a deletion change").unique_id();
                if area_db.has_entity(&osm_id)? {
                    Some(SemanticChange::removing(&osm_id))
                } else {
                    None
                }
            }
            Modify => {
                let osm_id = change
                    .old
                    .as_ref()
                    .unwrap_or(change.new.as_ref().expect("No old or new"))
                    .unique_id();
                let old = area_db.get_entity(&osm_id)?;
                let new = translator::translate(
                    &change.new.expect("No new entity during a modify"),
                    &manager,
                )?;
                match (old, new) {
                    (None, None) => None,
                    (Some(_), None) => Some(SemanticChange::removing(&osm_id)),
                    (None, Some(new)) => Some(SemanticChange::creating(
                        new.geometry,
                        new.discriminator,
                        new.data,
                        new.effective_width,
                    )),
                    (Some(old), Some(new)) => {
                        let (property_changes, data_changes) =
                            diff_utils::diff_entities(&old, &new)?;
                        Some(SemanticChange::updating(
                            &osm_id,
                            property_changes,
                            data_changes,
                        ))
                    }
                }
            }
        };
        if let Some(semantic_change) = semantic_change {
            area_db.apply_change(&semantic_change)?;
            tokio::run_async(area_messaging::publish_change(
                semantic_change,
                area.name.clone(),
            ));
        }
    }
    area.state = AreaState::Updated;
    area.save(*&conn)?;
    info!("Area updated successfully.");
    Ok(())
}

pub fn update_area_databases() -> Result<()> {
    info!("Going to perform the database update for all up-to date databases.");
    let area_db_conn = SqliteConnection::establish("server.db")?;
    for mut area in Area::all_updated(&area_db_conn)? {
        update_area(&mut area, &area_db_conn)?;
    }
    info!("Updates finished successfully.");
    BackgroundTask::UpdateAreaDatabases.deliver_at_time(DATABASES_UPDATE_HOUR, DATABASES_UPDATE_MINUTE, DATABASES_UPDATE_SECOND);
    Ok(())
}
