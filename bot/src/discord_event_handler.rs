use serenity::{async_trait, client::{EventHandler, Context}, model::channel::Message};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, _msg: Message) {
        
    }
}