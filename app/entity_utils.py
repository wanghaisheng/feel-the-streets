from osm_db import Enum
from .services import config

def filter_important_roads(roads):
    if not config().navigation.try_avoid_sidewalks:
        return roads
    non_sidewalks = [r for r in roads if r.value_of_field("type") != Enum.with_name("RoadType").value_for_name("footway")]
    if non_sidewalks:
        return non_sidewalks
    else:
        # When you're standing only on sidewalks, leaving one is okay regardless.
        return roads

def get_last_important_road(roads):
    if not config().navigation.try_avoid_sidewalks:
        return roads[-1]
    else:
        for candidate in reversed(roads):
            if candidate.value_of_field("type") != Enum.with_name("RoadType").value_for_name("footway"):
                return candidate
        # We guarantee only that we'll try to avoid them, not that we'll always skip them, so return the last road anyway.
        return roads[-1]