import json
import datetime
from pydantic import BaseModel
from .enums import OSMObjectType
from ..models import Entity
from ..humanization_utils import get_class_display_name

class OSMEntity(BaseModel):
    osm_id: str
    timestamp: datetime.datetime
    version: int
    changeset: int
    uid: int
    user: str
    parent_osm_id: str = None
    db_entity: Entity = None # Newer will be, but it allows the property processing logic to continue without marking all entities as missing a crucial property.

    class Config:
        allow_mutation = False
    
    def __hash__(self):
        return hash(repr(self))
    def __str__(self):
        return get_class_display_name(self.__class__)

    @property
    def additional_fields(self):
        names = set(self.__fields__.keys())
        props = json.loads(self.db_entity.data)
        unknown_props = {key: val for key, val in props.items() if key not in names}
        return unknown_props

    @property
    def effective_width(self):
        return None