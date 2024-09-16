use rspack_core::{
  AffectType, AsModuleDependency, Compilation, ContextDependency, ContextOptions,
  ContextTypePrefix, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, RealDependencyLocation, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

use super::{context_dependency_template_as_id, create_resource_identifier_for_context_dependency};

#[derive(Debug, Clone)]
pub struct RequireResolveContextDependency {
  id: DependencyId,
  options: ContextOptions,
  range: RealDependencyLocation,
  resource_identifier: String,
  optional: bool,
}

impl RequireResolveContextDependency {
  pub fn new(options: ContextOptions, range: RealDependencyLocation, optional: bool) -> Self {
    let resource_identifier = create_resource_identifier_for_context_dependency(None, &options);
    Self {
      id: DependencyId::new(),
      options,
      range,
      resource_identifier,
      optional,
    }
  }
}

impl Dependency for RequireResolveContextDependency {
  fn id(&self) -> DependencyId {
    self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireContext
  }

  fn range(&self) -> Option<&RealDependencyLocation> {
    Some(&self.range)
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }
}

impl ContextDependency for RequireResolveContextDependency {
  fn request(&self) -> &str {
    &self.options.request
  }

  fn options(&self) -> &ContextOptions {
    &self.options
  }

  fn get_context(&self) -> Option<&str> {
    None
  }

  fn resource_identifier(&self) -> &str {
    &self.resource_identifier
  }

  fn set_request(&mut self, request: String) {
    self.options.request = request;
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn type_prefix(&self) -> ContextTypePrefix {
    ContextTypePrefix::Normal
  }
}

impl DependencyTemplate for RequireResolveContextDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    context_dependency_template_as_id(self, source, code_generatable_context, &self.range);
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}

impl AsModuleDependency for RequireResolveContextDependency {}
