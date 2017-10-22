import enum
from sqlalchemy import Column, ForeignKeyConstraint, Boolean, Integer, UnicodeText
from ..sa_types import IntEnum
from . import Named
from .enums import OSMObjectType

class RouteType(enum.Enum):
    hiking = 1
    bicycle = 2
    tracks = 3
    train = 4
    road = 5
    power = 6
    foot = 7
    

class RouteImportance(enum.Enum):
    major = 0
    local = 1
    learning = 3
    ruin = 4
    peak = 5
    spring = 6
    interesting_object = 7
    horse = 8
    ski = 9
    bicycle = 10
    wheelchair = 11

class Route(Named):
    __tablename__ = "routes"
    __table_args__ = (ForeignKeyConstraint(["id", "osm_type"], ["named.id", "named.osm_type"]),)
    __mapper_args__ = {'polymorphic_identity': 'route'}
    id = Column(Integer, primary_key=True)
    osm_type = Column(IntEnum(OSMObjectType), primary_key=True)
    type = Column(IntEnum(RouteType), nullable=False)
    complete = Column(Boolean)
    destinations = Column(UnicodeText)
    kct_red = Column(IntEnum(RouteImportance))
    network = Column(UnicodeText)
    note = Column(UnicodeText)
    operator = Column(UnicodeText)
    symbol = Column(UnicodeText)
    kct_yellow = Column(IntEnum(RouteImportance))
    kct_green = Column(IntEnum(RouteImportance))
    kct_blue = Column(IntEnum(RouteImportance))
    alt_name = Column(UnicodeText)
    description = Column(UnicodeText)
    from_ = Column(UnicodeText)
    kct_none = Column(IntEnum(RouteImportance))
    wikidata = Column(UnicodeText)
    wikipedia = Column(UnicodeText)