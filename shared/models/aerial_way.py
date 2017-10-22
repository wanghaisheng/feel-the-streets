import enum
from sqlalchemy import Column, Boolean, ForeignKeyConstraint, Integer, UnicodeText
from ..sa_types import IntEnum
from .enums import AerialWayType, RoadType, RoofShape, BridgeStructure, OSMObjectType
from . import Named

class AerodromeType(enum.Enum):
    private = 0

class AerialWay(Named):
    __tablename__ = "aerialways"
    __table_args__ = (ForeignKeyConstraint(["id", "osm_type"], ["named.id", "named.osm_type"]),)
    __mapper_args__ = {'polymorphic_identity': 'aerialway'}
    id = Column(Integer, primary_key=True)
    osm_type = Column(IntEnum(OSMObjectType), primary_key=True)
    type = Column(IntEnum(AerialWayType), nullable=False)
    capacity = Column(Integer)
    occupancy = Column(Integer)
    duration = Column(Integer)
    wikipedia = Column(UnicodeText)
    building = Column(UnicodeText)
    length = Column(Integer)
    width = Column(Integer)
    heading = Column(UnicodeText)
    surface = Column(UnicodeText)
    aerodrome_type = Column(IntEnum(AerodromeType))
    icao = Column(UnicodeText)
    roof_shape = Column(IntEnum(RoofShape))
    heating = Column(Boolean)
    pedestrian = Column(Boolean)
    highway = Column(IntEnum(RoadType))
    bridge_structure = Column(IntEnum(BridgeStructure))
    bridge = Column(Boolean)
    wikidata = Column(UnicodeText)