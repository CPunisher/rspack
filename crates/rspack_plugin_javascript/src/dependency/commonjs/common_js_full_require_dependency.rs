use rspack_core::{
  module_id, property_access, to_normal_comment, Compilation, ExportsType,
  ExtendedReferencedExport, ModuleGraph, RealDependencyLocation, RuntimeGlobals, RuntimeSpec,
  UsedName,
};
use rspack_core::{AsContextDependency, Dependency, DependencyCategory};
use rspack_core::{DependencyId, DependencyTemplate};
use rspack_core::{DependencyType, ModuleDependency};
use rspack_core::{TemplateContext, TemplateReplaceSource};
use swc_core::atoms::Atom;

#[derive(Debug, Clone)]
pub struct CommonJsFullRequireDependency {
  id: DependencyId,
  request: String,
  names: Vec<Atom>,
  range: RealDependencyLocation,
  is_call: bool,
  optional: bool,
  asi_safe: bool,
}

impl CommonJsFullRequireDependency {
  pub fn new(
    request: String,
    names: Vec<Atom>,
    range: RealDependencyLocation,
    is_call: bool,
    optional: bool,
    asi_safe: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      names,
      range,
      is_call,
      optional,
      asi_safe,
    }
  }
}

impl Dependency for CommonJsFullRequireDependency {
  fn id(&self) -> DependencyId {
    self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsRequire
  }

  fn loc(&self) -> Option<String> {
    Some(self.range.to_string())
  }

  fn range(&self) -> Option<&RealDependencyLocation> {
    Some(&self.range)
  }

  fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if self.is_call
      && module_graph
        .module_graph_module_by_dependency_id(self.id)
        .and_then(|mgm| module_graph.module_by_identifier(&mgm.module_identifier))
        .map(|m| m.get_exports_type(module_graph, false))
        .is_some_and(|t| !matches!(t, ExportsType::Namespace))
    {
      if self.names.is_empty() {
        return vec![ExtendedReferencedExport::Array(vec![])];
      } else {
        return vec![ExtendedReferencedExport::Array(
          self.names[0..self.names.len() - 1].to_vec(),
        )];
      }
    }
    vec![ExtendedReferencedExport::Array(self.names.clone())]
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

impl ModuleDependency for CommonJsFullRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn get_optional(&self) -> bool {
    self.optional
  }
}

impl DependencyTemplate for CommonJsFullRequireDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      runtime,
      runtime_requirements,
      ..
    } = code_generatable_context;
    let module_graph = compilation.get_module_graph();
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);

    let mut require_expr = format!(
      r#"{}({})"#,
      RuntimeGlobals::REQUIRE,
      module_id(compilation, self.id, &self.request, false)
    );

    if let Some(imported_module) = module_graph.module_graph_module_by_dependency_id(self.id) {
      let used = module_graph
        .get_exports_info(&imported_module.module_identifier)
        .get_used_name(&module_graph, *runtime, UsedName::Vec(self.names.clone()));

      if let Some(used) = used {
        let comment = to_normal_comment(&property_access(self.names.clone(), 0));
        require_expr = format!(
          "{}{}{}",
          require_expr,
          comment,
          property_access(
            match used {
              UsedName::Str(name) => vec![name].into_iter(),
              UsedName::Vec(names) => names.into_iter(),
            },
            0
          )
        );
        if self.asi_safe {
          require_expr = format!("({require_expr})");
        }
      }
    }

    source.replace(self.range.start, self.range.end, &require_expr, None);
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

impl AsContextDependency for CommonJsFullRequireDependency {}
