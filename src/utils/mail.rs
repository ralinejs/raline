use anyhow::Context;
use askama::Template;
use spring_mail::{header::ContentType, AsyncTransport, Mailer, Message};
use spring_web::error::Result;

pub async fn send_mail<T: Template>(
    mailer: &Mailer,
    from: &str,
    to: &str,
    subject: &str,
    body: &T,
) -> Result<bool> {
    let from_mail_box = from
        .parse()
        .with_context(|| format!("email {} is invalid", from))?;
    let to_mailbox = to
        .parse()
        .with_context(|| format!("email {} is invalid", to))?;
    let content_type = ContentType::parse(T::MIME_TYPE)
        .with_context(|| format!("content type parse failed: {}", T::MIME_TYPE))?;
    let body = body.render().context("template render failed")?;
    let message = Message::builder()
        .from(from_mail_box)
        .to(to_mailbox)
        .subject(subject)
        .header(content_type)
        .body(body)
        .context("mail build error")?;

    let resp = mailer
        .send(message)
        .await
        .with_context(|| format!("send mail to {to} failed"))?;

    Ok(resp.is_positive())
}