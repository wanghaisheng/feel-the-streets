from .address_aware import AddressAwareGenerator
from shared.models import RailWay

class RailWayGenerator(AddressAwareGenerator):
    def __init__(self):
        super().__init__()
        self.generates(RailWay)
        self.renames("railway", "type")
        self.renames("bridge", "is_bridge")
        self.renames("crossing:light", "crossing_light")
        self.renames("crossing:bell", "crossing_bell")
        self.renames("crossing:barrier", "crossing_barrier")
        self.renames("crossing", "crossing_type")
        self.renames("building", "building_type")
        self.renames("oneway:tram", "tram_oneway")
        self.renames("maxspeed:tilting", "tilting_maxspeed")
        self.renames("abandoned:railway", "abandoned_railway")
        self.renames("bridge:name", "bridge_name")
        self.renames("bridge:structure", "bridge_structure")
        self.renames("disused:railway", "disused_railway")
        self.renames("tunnel:name", "tunnel_name")
        self.renames("indoor:level", "indoor_level")
        self.renames("maxspeed:forward", "forward_maxspeed")
        self.renames("maxspeed:backward", "backward_maxspeed")
        self.renames("railway:position:exact", "exact_position")
        self.renames("railway:signal:direction", "signal_direction")
        self.renames("railway:signal:position", "signal_position")
        self.renames("railway:signal:shunting", "shunting_signal")
        self.renames("railway:signal:shunting:form", "shunting_signal_form")
        self.renames("railway:signal:shunting:height", "shunting_signal_height")
        self.renames("railway:signal:main_repeated:form", "main_repeated_signal_form")
        self.renames("railway:signal:main_repeated:states", "main_repeated_signal_states")
        self.renames("railway:signal:distant", "distant_signal")
        self.renames("railway:signal:distant:form", "distant_signal_form")
        self.renames("railway:signal:distant:height", "distant_signal_height")
        self.renames("railway:signal:distant:states", "distant_signal_states")
        self.renames("railway:signal:main", "main_signal")
        self.renames("railway:signal:main:form", "main_signal_form")
        self.renames("railway:signal:main:function", "main_signal_function")
        self.renames("railway:signal:main:height", "main_signal_height")
        self.renames("railway:signal:main:states", "main_signal_states")
        self.renames("railway:signal:main:substitute_signal", "main_signal_substitute_signal")
        self.renames("railway:signal:speed_limit_distant", "distant_speed_limit_signal")
        self.renames("railway:signal:speed_limit_distant:form", "distant_speed_limit_signal_form")
        self.renames("railway:signal:speed_limit_distant:height", "distant_speed_limit_signal_height")
        self.renames("railway:signal:speed_limit_distant:speed", "distant_speed_limit_signal_speed")
        self.renames("railway:signal:electricity", "electricity_signal")
        self.renames("railway:signal:electricity:form", "electricity_signal_form")
        self.renames("railway:signal:electricity:height", "electricity_signal_height")
        self.renames("railway:signal:shunting:states", "shunting_signal_states")
        self.renames("railway:signal:speed_limit", "speed_limit_signal")
        self.renames("railway:signal:speed_limit:form", "speed_limit_signal_form")
        self.renames("railway:signal:speed_limit:height", "speed_limit_signal_height")
        self.renames("railway:signal:speed_limit:speed", "speed_limit_signal_speed")
        self.renames("crossing:activation", "crossing_activation")
        self.renames("crossing:saltire", "crossing_saltire")
        self.renames("railway:signal:distant:deactivated", "distant_signal_deactivated")
        self.renames("railway:signal:main:deactivated", "main_signal_deactivated")
        self.renames("railway:signal:shunting:deactivated", "shunting_signal_deactivated")
        self.renames("railway:signal:main_repeated:deactivated", "main_repeated_signal_deactivated")
        self.renames("railway:signal:station_distant", "distant_station_signal")
        self.renames("railway:signal:station_distant:form", "distant_station_signal_form")
        self.renames("railway:signal:snowplow", "snowplow_signal")
        self.renames("railway:signal:snowplow:form", "snowplow_signal_form")
        self.renames("railway:signal:snowplow:type", "snowplow_signal_type")
        self.renames("railway:maxspeed:diverging", "diverging_maxspeed")
        self.renames("railway:maxspeed:straight", "straight_maxspeed")
        self.renames("railway:radius", "gauge")
        self.renames("railway:signal:crossing_distant", "distant_crossing_signal")
        self.renames("railway:signal:crossing_distant:form", "distant_crossing_signal_form")
        self.renames("railway:signal:main_repeated:substitute_signal", "main_repeated_signal_substitute_signal")
        self.renames("railway:signal:crossing_distant:deactivated", "distant_crossing_signal_deactivated")
        self.renames("railway:signal:crossing_distant:caption", "distant_crossing_signal_caption")
        self.renames("railway:signal:snowplow:deactivated", "snowplow_signal_deactivated")
        self.renames("railway:signal:minor", "minor_signal")
        self.renames("railway:signal:minor:deactivated", "minor_signal_deactivated")
        self.renames("railway:signal:minor:form", "minor_signal_form")
        self.renames("railway:signal:minor:height", "minor_signal_height")
        self.renames("railway:switch:electric", "electric_switch")
        self.renames("railway:signal:minor:substitute_signal", "minor_signal_substitute_signal")
        self.renames("railway:signal:speed_limit:deactivated", "speed_limit_signal_deactivatedd")
        self.renames("railway:signal:electricity:deactivated", "electricity_signal_deactivated")
        self.renames("railway:signal:station_distant:deactivated", "distant_station_signal_deactivated")
        self.renames("railway:signal:minor:states", "minor_signal_states")
        self.renames("avalanche_protector:left", "left_avalanche_protector")
        self.renames("milestone:catenary_mast", "catenary_mast_milestone")
        self.renames("milestone:emergency_brake_override", "emergency_brake_override_milestone")
        self.renames("railway:defect_detector:hot_box", "hot_box_defect_detector")
        self.removes_subtree("ruian")
        self.removes("maxspeed:source")
        self.unprefixes("building")
        self.unprefixes("railway")
        self.removes("railway:ref", True)
        self.removes("ref")
        self.removes("uic_ref")
    @staticmethod
    def accepts(props):
        return "railway" in props