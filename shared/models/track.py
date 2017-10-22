from sqlalchemy import Column, ForeignKeyConstraint, Boolean, Integer, UnicodeText
from ..sa_types import IntEnum
from . import Road
from .enums import Inclination, LeisureType, SportType, AccessType, OSMObjectType

class Track(Road):
    __tablename__ = "tracks"
    __table_args__ = (ForeignKeyConstraint(["id", "osm_type"], ["roads.id", "roads.osm_type"]),)
    __mapper_args__ = {'polymorphic_identity': 'track'}
    id = Column(Integer, primary_key=True)
    osm_type = Column(IntEnum(OSMObjectType), primary_key=True)
    motorcar_allowed = Column(IntEnum(AccessType))
    leisure = Column(IntEnum(LeisureType))
    sport = Column(IntEnum(SportType))
    website = Column(UnicodeText)
    mtb_scale = Column(Integer)
    mtb_scale_uphill = Column(Integer)