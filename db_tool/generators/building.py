from .address_aware import AddressAwareGenerator
from shared.models import Building

class BuildingGenerator(AddressAwareGenerator):
    def __init__(self):
        super().__init__()
        self.generates(Building)
        self.unprefixes("building")
        self.renames("roof:height", "roof_height")
        self.renames("roof:shape", "roof_shape")
        self.renames("building:roof:shape", "roof_shape")
        self.renames("building:roof:angle", "roof_angle")
        self.renames("roof:orientation", "roof_orientation")
        self.renames("building:roof:orientation", "roof_orientation")
        self.renames("internet_access:fee", "internet_access_fee")
        self.renames("information", "information_type")
        self.renames("webpage", "website")
        self.renames("url", "website")
        self.renames("diet:vegetarian", "vegetarian_diet")
        self.renames("heritage:operator", "heritage_operator")
        self.renames("payment:visa", "visa_payment")
        self.renames("payment:visa_debit", "visa_debit_payment")
        self.renames("payment:visa_electron", "visa_electron_payment")
        self.renames("roof:levels", "roof_levels")
        self.renames("roof:material", "roof_material")
        self.renames("roof", "roof_shape")
        self.renames("building:roof", "roof_shape")
        self.renames("wheelchair:description", "wheelchair_description")
        self.removes("building")
        self.removes("type")
        self.removes_subtree("ref")
        self.removes_subtree("ruian")
        self.removes_subtree("isced")
        self.removes_subtree("uir_adr")
        self.removes_subtree("wikipedia")
        self.removes_subtree("old_name")
        self.removes_subtree("alt_name")
        self.removes("ruian")
        self.renames("community_centre:for", "community_centre_for")
        self.renames("leisure", "leisure_type")
        self.renames("tourism", "tourism_type")
        self.renames("industrial", "industrial_type")
        self.renames("building:levels:underground", "underground_levels")
        self.renames("roof:angle", "roof_angle")
        self.renames("roof:colour", "roof_colour")
        self.renames("toilets:wheelchair", "wheelchair_toilets")
        self.renames("healthcare:speciality", "healthcare_speciality") # It should go out 
        self.renames("wheelchair:toilets", "wheelchair_toilets")
        self.renames("ph", "phone")
        self.renames("opening_hours:url", "opening_hours_url")
        
        self.unprefixes("contact")
        self.renames("historic", "historic_type")

        

    @staticmethod
    def accepts(props):
        return ("building" in props and "aerialway" not in props) or "shop" in props or "building:levels" in props or ("amenity" in props and props["amenity"] in {"kindergarten", "school", "college", "hospital", "restaurant", "doctors", "veterinary", "dentist", "clinic"}) or ("type" in props and props["type"] == "building")