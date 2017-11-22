import sys
import time
import logging
import geoalchemy.functions as gf
from geoalchemy import WKTSpatialElement
import shapely.wkt
from shared.database import Database
from .osm_object_translator import OSMObjectTranslator
from .utils import get_srid_from_point

log = logging.getLogger(__name__)

class DatabaseUpdater:
    def __init__(self, location):
        self._location = location
        self.translator = OSMObjectTranslator()
        self._db = Database(location)
        self._db.create_if_needed()
        self._check_geoms = "--check-geometries" in sys.argv

    def update_database(self):
        entities = []
        for entity, object in self.translator.translated_objects_in(self._location):
            geometry = self.translator.manager.get_geometry_as_wkt(object)
            if not geometry:
                log.error("Failed to generate geometry for %s.", object)
                continue
            if self._check_geoms:
                try:
                    shapely.wkt.loads(geometry)
                except Exception as exc:
                    log.error("Failed to parse geometry of %s, error %s.", object, exc)
                    continue
            entity.geometry = geometry
            self._db.add(entity)
            self._maybe_polygonize(entity)
        log.info("Committing changes.")
        self._db.commit()
        self._db.commit()

    def _maybe_polygonize(self, entity):
        if hasattr(entity, "effective_width") and entity.effective_width > 0 and self._db.scalar(gf.geometry_type(entity.geometry)) == "LINESTRING":
            entity.original_geometry = entity.geometry
            log.debug("Creating containment polygon for entity %s.", entity.id)
            x = self._db.scalar(entity.original_geometry.point_n(1).x())
            y = self._db.scalar(entity.original_geometry.point_n(1).y())
            entity.geometry = WKTSpatialElement(self._db.scalar(entity.original_geometry.transform(get_srid_from_point(x, y)).buffer(entity.effective_width).transform(4326).wkt()))
        return entity

