pub mod model;

use whisperfish::gui::QmlAppPromptApi;
use crate::model::prompt::PromptBox;

impl QmlAppPromptApi for PromptBox {}