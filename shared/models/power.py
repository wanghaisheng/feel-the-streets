import enum
from sqlalchemy import Column, ForeignKeyConstraint, Integer, UnicodeText
from ..sa_types import IntEnum
from .entity import Entity
from .enums import OSMObjectType, BuildingType, Location

class PowerType(enum.Enum):
    tower = 0
    substation = 1
    portal = 2
    station = 3
    transformer = 4
    cable_distribution_cabinet = 5
    minor_line = 6
    cable = 7
    insulator = 8
    terminal = 9
    switch = 10
    transition = 11
    sub_station = 12
    switchgear = 13
    pole = 14
    

class Power(Entity):
    __tablename__ = "powers"
    __table_args__ = (ForeignKeyConstraint(["id", "osm_type"], ["entities.id", "entities.osm_type"]),)
    __mapper_args__ = {'polymorphic_identity': 'power'}
    id = Column(Integer, primary_key=True)
    osm_type = Column(IntEnum(OSMObjectType), primary_key=True)
    type = Column(IntEnum(PowerType), nullable=False)
    building = Column(IntEnum(BuildingType))
    layer = Column(Integer)
    fixme = Column(UnicodeText)
    location = Column(IntEnum(Location))
    high_voltage = Column(Integer)
    voltage = Column(Integer)
    low_voltage = Column(Integer)