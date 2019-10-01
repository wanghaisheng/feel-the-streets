import wx
from osm_db import EntityMetadata, all_known_discriminators, EntitiesQuery
from shared.humanization_utils import get_class_display_name
from shared.geometry_utils import xy_ranges_bounding_square
from .search_conditions import SpecifySearchConditionsDialog
from ..geometry_utils import distance_filter
from ..services import map

from uimanager import get

def perform_search(position):
    entities = all_known_discriminators()
    name_mapping = {}
    for entity in entities:
        name_mapping[get_class_display_name(entity)] = entity
    entity = wx.GetSingleChoice(_("Select the class to search"), _("Search class"), aChoices=sorted(name_mapping.keys()))
    if entity:
        discriminator = name_mapping[entity]
        conditions_dialog = get().prepare_xrc_dialog(SpecifySearchConditionsDialog, entity=discriminator, alternate_bind_of=["on_fields_tree_sel_changed"])
        if conditions_dialog.ShowModal() != wx.ID_OK:
            conditions_dialog.Destroy()
            return
        conditions = conditions_dialog.create_conditions()
        distance = float("inf")
        query = EntitiesQuery()
        if conditions_dialog.distance:
            distance = conditions_dialog.distance
            min_x, min_y, max_x, max_y = xy_ranges_bounding_square(position, distance*2)
            query.set_rectangle_of_interest(min_x, max_x, min_y, max_y)
        discriminators = []
        metadata = EntityMetadata.for_discriminator(discriminator)
        while metadata:
            discriminators.append(metadata.discriminator)
            metadata = metadata.parent_metadata
        query.set_included_discriminators(discriminators)
        for condition in conditions:
            query.add_condition(condition)
        results = map()._db.get_entities(query)
        filtered_objects = distance_filter(results, position, distance)
        conditions_dialog.Destroy()
        return filtered_objects
    
    