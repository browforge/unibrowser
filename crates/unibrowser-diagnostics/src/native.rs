use crate::dictionary::DictionarySnapshot;
use crate::host::TextCheckContext;
use crate::problem::Problem;

pub trait NativeDiagnosticsChecker: Send + Sync {
    fn checker_id(&self) -> &str;

    fn check_text(
        &self,
        _doc_uri: &str,
        _text: &str,
        _ctx: &TextCheckContext,
        _dict: &DictionarySnapshot,
    ) -> Vec<Problem> {
        Vec::new()
    }

    fn check_dxf(&self, _path: &str, _content: &str) -> Vec<Problem> {
        Vec::new()
    }

    fn check_unibrow(&self, _project_json: &str) -> Vec<Problem> {
        Vec::new()
    }
}
