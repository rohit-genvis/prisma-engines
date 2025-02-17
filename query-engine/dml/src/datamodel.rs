use crate::composite_type::CompositeType;
use crate::field::RelationField;
use crate::model::Model;
use crate::relation_info::RelationInfo;

/// Entities in the datamodel can be flagged as `is_commented_out`. This lets the renderer
/// know that introspection encountered unsupported names or features and these are supposed
/// to be rendered as comments. Since the parser will not set these flags when reading a schema
/// string, only introspection and the lowering of the datamodel to the ast care about these flags.
/// The FieldType: Unsupported behaves in the same way.
/// Both of these are never converted into the internal datamodel.
#[derive(Debug, Default)]
pub struct Datamodel {
    pub models: Vec<Model>,
    pub composite_types: Vec<CompositeType>,
}

impl Datamodel {
    /// Gets an iterator over all models.
    pub fn models(&self) -> std::slice::Iter<Model> {
        self.models.iter()
    }

    /// Gets an iterator over all composite types.
    pub fn composite_types(&self) -> std::slice::Iter<CompositeType> {
        self.composite_types.iter()
    }

    /// Finds a model by name.
    pub fn find_model(&self, name: &str) -> Option<&Model> {
        self.models().find(|model| model.name == name)
    }

    /// Finds a composite type by name.
    pub fn find_composite_type(&self, name: &str) -> Option<&CompositeType> {
        self.composite_types().find(|composite| composite.name == name)
    }

    /// Finds a model by database name. This will only find models with a name
    /// remapped to the provided `db_name`.
    pub fn find_model_db_name(&self, db_name: &str) -> Option<&Model> {
        self.models()
            .find(|model| model.database_name.as_deref() == Some(db_name))
    }

    /// Finds parent  model for a field reference.
    pub fn find_model_by_relation_field_ref(&self, field: &RelationField) -> Option<&Model> {
        self.find_model(&self.find_related_field_bang(field).1.relation_info.referenced_model)
    }

    /// Finds a model by name and returns a mutable reference.
    pub fn find_model_mut(&mut self, name: &str) -> &mut Model {
        self.models
            .iter_mut()
            .find(|m| m.name == *name)
            .expect("We assume an internally valid datamodel before mutating.")
    }

    /// Returns (model_name, field_name) for all relation fields pointing to a specific model.
    pub fn find_relation_fields_for_model(&mut self, model_name: &str) -> Vec<(String, String)> {
        let mut fields = vec![];
        for model in self.models() {
            for field in model.relation_fields() {
                if field.relation_info.referenced_model == model_name {
                    fields.push((model.name.clone(), field.name.clone()))
                }
            }
        }
        fields
    }

    /// Finds a relation field related to a relation info. Returns a tuple (index_of_relation_field_in_model, relation_field).
    pub fn find_related_field_for_info(&self, info: &RelationInfo, exclude: &str) -> Option<(usize, &RelationField)> {
        self.find_model(&info.referenced_model)
            .expect("The model referred to by a RelationInfo should always exist.")
            .fields
            .iter()
            .enumerate()
            .filter_map(|(idx, field)| field.as_relation_field().map(|f| (idx, f)))
            .find(|(_idx, f)| {
                f.relation_info.name == info.name
                    && (f.relation_info.referenced_model != info.referenced_model ||
          // This is to differentiate the opposite field from self in the self relation case.
          f.name != exclude)
            })
    }

    /// This finds the related field for a relationfield if available
    pub fn find_related_field(&self, rf: &RelationField) -> Option<(usize, &RelationField)> {
        self.find_related_field_for_info(&rf.relation_info, &rf.name)
    }

    /// This is used once we assume the datamodel to be internally valid
    pub fn find_related_field_bang(&self, rf: &RelationField) -> (usize, &RelationField) {
        self.find_related_field(rf)
            .expect("Every RelationInfo should have a complementary RelationInfo on the opposite relation field.")
    }
}
