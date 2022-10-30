use async_trait::async_trait;
use crate::service::SharedServiceApi;

pub trait InitializeWithService {
    fn initialize_with_service(&self, app: SharedServiceApi);
}

pub trait RegisterIn<T> {
    fn register_in(&self, target: &mut T);
}

#[async_trait(?Send)]
pub trait PromptApi {
    async fn ask_phone_number(&self) -> Option<String>;
    async fn ask_verification_code(&self) -> Option<String>;
    async fn ask_captcha(&self) -> Option<String>;
    async fn ask_registration_type(&self) -> Option<bool>;
    async fn ask_password(&self) -> Option<String>;
    fn show_link_qr(&self, url: String);
}

#[async_trait(?Send)]
pub trait SetupWorkerApi {

}