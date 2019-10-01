import re
import builtins
from osm_db import EntityMetadata, Enum
import jinja2
import logging

known_enums = Enum.all_known()
jinja2_env = jinja2.Environment(loader=jinja2.BaseLoader())

log = logging.getLogger(__name__)

def underscored_to_words(underscored):
    _ = getattr(builtins, "_", lambda s: s)
    if not underscored:
        underscored = "unknown"
    underscored = underscored.replace("_", " ")
    underscored = re.sub(r"^\w", lambda m: m.group(0).upper(), underscored)
    return _(underscored)

def get_field_type(key, fields):
    if key in fields:
        return fields[key].type_name
    else:
        return "str"
    

def format_field_value(field_value, field_type):
    if field_type in known_enums:
        try:
            if isinstance(field_value, str):
                log.warn("Field value %s of the enum %s expected to be an int.", field_value, field_type)
                return underscored_to_words(field_value)
            return underscored_to_words(Enum.with_name(field_type).name_for_value(field_value))
        except ValueError: pass
    try:
        metadata = EntityMetadata.for_discriminator(field_type)
    except KeyError:
        return field_value
    return describe_nested_object(field_value, metadata)
    
def format_class_name(name):
    return re.sub(r"([a-z\d])([A-Z])([a-z\d])", lambda m: "%s %s%s"%(m.group(1), m.group(2).lower(), m.group(3)), name)

def get_class_display_name(klass):
    _ = getattr(builtins, "_", lambda s: s)
    return _(format_class_name(klass))

def describe_entity(entity, metadata=None):
    template = None
    if not metadata:
        metadata = EntityMetadata.for_discriminator(entity.discriminator)
    search_target = metadata
    while not template:
        template = search_target.display_template
        search_target = search_target.parent_metadata
    context = {}
    fields = metadata.all_fields
    for field_name in entity.defined_field_names:
        if field_name not in fields:
            continue
        value = entity.value_of_field(field_name)
        context[field_name] = format_field_value(value, fields[field_name].type_name)
    # Special variables
    if search_target:
        context["parent_display"] = describe_entity(entity, search_target)
    context["class_name_display"] = get_class_display_name(entity.discriminator)
    template_object = jinja2_env.from_string(template)
    return template_object.render(**context)

def describe_nested_object(obj, metadata):
    context = {}
    for key, val in obj.items():
        context[key] = format_field_value(val, metadata.fields[key].type_name)
    return jinja2_env.from_string(metadata.display_template).render(**context)