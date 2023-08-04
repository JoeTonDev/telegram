use teloxide::{
    dispatching::{ dialogue::GetChatId, dialogue, dialogue::InMemStorage, UpdateHandler },
    prelude::*,
    types::{ InlineKeyboardButton, InlineKeyboardMarkup, ForceReply, ReplyMarkup },
    utils::command::BotCommands,
    prelude::*,
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
// type MyStorage = std::sync::Arc<ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Start")]
    Start,
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    Symbol,
    Period {
        symbol: String,
    },
    List {
        symbol: String,
        period: String,
    },
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting buttons bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch().await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide
        ::filter_command::<Command, _>()
        .branch(case![State::Start].branch(case![Command::Start].endpoint(start)));

    let message_handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(command_handler);

    let callback_query_handler = Update::filter_callback_query()
        .branch(case![State::List { period, symbol }].endpoint(list))
        .branch(case![State::Period { symbol }].endpoint(period))
        .branch(case![State::Symbol].endpoint(symbol));

    dialogue
        ::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

async fn start(bot: Bot, msg: Message, dialogue: MyDialogue, q: CallbackQuery) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(text) => {
            let symbol_button = ["BTCUSDT", "ETHUSDT"].map(|symbol|
                InlineKeyboardButton::callback(symbol, symbol)
            );
            bot.answer_callback_query(q.id).await?;
            bot
                .send_message(dialogue.chat_id(), "Select your trading pair: ")
                .reply_markup(InlineKeyboardMarkup::new([symbol_button])).await?;
            dialogue.update(State::Symbol).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please select:").await?;
        }
    }

    Ok(())
}

async fn symbol(bot: Bot, dialogue: MyDialogue, q: CallbackQuery) -> HandlerResult {
    if let Some(symbol) = &q.data {
        let interval_button = ["1h", "4h", "1d"].map(|interval|
            InlineKeyboardButton::callback(interval, interval)
        );
        bot.answer_callback_query(q.id).await?;
        bot
            .send_message(dialogue.chat_id(), format!("Select interval for {symbol}:"))
            .reply_markup(InlineKeyboardMarkup::new([interval_button])).await?;
        dialogue.update(State::Period { symbol: symbol.to_string() }).await?;
    } else {
        bot.send_message(dialogue.chat_id(), "Select symbol").await?;
    }

    Ok(())
}

async fn period(bot: Bot, dialogue: MyDialogue, q: CallbackQuery, symbol: String) -> HandlerResult {
    if let Some(period) = &q.data {
        let submit_button = ["Submit"].map(|submit| InlineKeyboardButton::callback(submit, submit));

        bot.answer_callback_query(q.id).await?;
        bot
            .send_message(dialogue.chat_id(), format!("Ticker: {symbol}\nInterval: {period}"))
            .reply_markup(InlineKeyboardMarkup::new([submit_button])).await?;
        dialogue.update(State::List {
            symbol: symbol.to_string(),
            period: period.to_string(),
        }).await?;
    } else {
        bot.send_message(dialogue.chat_id(), "Select interval").await?;
    }

    Ok(())
}

async fn list(
    bot: Bot,
    dialogue: MyDialogue,
    period: String,
    symbol: String,
    q: CallbackQuery
) -> HandlerResult {
    if let Some(data) = &q.data {
        bot.send_message(dialogue.chat_id(), format!("{period}\n {symbol}")).await?;
    }

    Ok(())
}
