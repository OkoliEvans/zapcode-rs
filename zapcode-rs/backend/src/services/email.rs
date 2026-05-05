use crate::models::{Merchant, Transaction};
use anyhow::{anyhow, Context, Result};
use lettre::message::{header::ContentType, Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

fn build_welcome_html(business_name: &str, wallet_address: &str) -> String {
    format!(
        r#"
  <div style="font-family:system-ui,sans-serif;max-width:520px;margin:0 auto;background:#060912;color:#eef1ff;padding:36px;border-radius:12px;">
    <div style="font-size:11px;letter-spacing:.2em;text-transform:uppercase;color:#4d7cfe;margin-bottom:8px;">Zapcode</div>
    <h1 style="font-size:28px;margin:0 0 6px;color:#eef1ff;">Welcome, {} 👋</h1>
    <p style="color:#8fa4c8;font-size:13px;margin:0 0 28px;">Your USDC payment QR code is ready. Start accepting payments immediately.</p>

    <div style="background:#111828;border:1px solid rgba(255,255,255,.1);border-radius:10px;padding:20px;margin-bottom:20px;">
      <div style="font-size:10px;letter-spacing:.15em;text-transform:uppercase;color:#8fa4c8;margin-bottom:6px;">Your wallet address</div>
      <div style="font-family:monospace;font-size:12px;color:#4d7cfe;word-break:break-all;line-height:1.6;">{}</div>
    </div>

    <p style="font-size:13px;color:#8fa4c8;line-height:1.8;">
      Your QR code is available in your dashboard. Download it, print it, and place it anywhere — your counter, tables, invoices, or website.<br/><br/>
      Zapcode never holds your funds. Your wallet, your keys.
    </p>

    <p style="margin-top:28px;font-size:12px;color:#6b7ea8;text-align:center;letter-spacing:.02em;">
      Zapcode · Non-custodial USDC payments on Starknet
    </p>
  </div>"#,
        business_name, wallet_address,
    )
}

fn build_payment_html(merchant: &Merchant, tx: &Transaction, fiat_amount: Option<f64>) -> String {
    let explorer_base = if merchant.network == "mainnet" {
        "https://voyager.online/tx"
    } else {
        "https://sepolia.voyager.online/tx"
    };
    let fiat_info = if let Some(amount) = fiat_amount {
        format!(
            "<div style=\"font-size:13px;color:#8fa4c8;margin-top:4px;\">≈ {:.2} {}</div>",
            amount, merchant.currency
        )
    } else {
        "".to_string()
    };

    format!(
        r#"
  <div style="font-family:system-ui,sans-serif;max-width:520px;margin:0 auto;background:#060912;color:#eef1ff;padding:36px;border-radius:12px;">
    <div style="font-size:11px;letter-spacing:.2em;text-transform:uppercase;color:#4d7cfe;margin-bottom:8px;">Zapcode</div>
    <h1 style="font-size:28px;margin:0 0 6px;color:#eef1ff;">Payment received ✓</h1>
    <p style="color:#8fa4c8;font-size:13px;margin:0 0 28px;">A USDC payment just landed on your wallet.</p>

    <div style="background:#111828;border:1px solid rgba(255,255,255,.1);border-radius:10px;padding:20px;margin-bottom:20px;">
      <div style="font-size:36px;font-weight:700;color:#4d7cfe;">{:.2} USDC</div>
      {}
    </div>

    <table style="width:100%;font-size:13px;border-collapse:collapse;">
      <tr><td style="color:#8fa4c8;padding:6px 0;">Business</td><td style="text-align:right;color:#eef1ff;">{}</td></tr>
      <tr><td style="color:#8fa4c8;padding:6px 0;">From</td><td style="text-align:right;color:#eef1ff;font-family:monospace;font-size:11px;">{}…{}</td></tr>
      <tr><td style="color:#8fa4c8;padding:6px 0;">Network</td><td style="text-align:right;color:#eef1ff;">Starknet {}</td></tr>
    </table>

    <a href="{}/{}" style="display:block;margin-top:24px;padding:14px;background:#4d7cfe;color:#ffffff;text-align:center;border-radius:8px;text-decoration:none;font-size:12px;font-weight:700;letter-spacing:.08em;text-transform:uppercase;">
      View on Voyager →
    </a>

    <p style="margin-top:28px;font-size:12px;color:#6b7ea8;text-align:center;letter-spacing:.02em;">
      Zapcode · Non-custodial USDC payments on Starknet
    </p>
  </div>"#,
        tx.amount,
        fiat_info,
        merchant.business_name,
        &tx.from_address[..14],
        &tx.from_address[tx.from_address.len().saturating_sub(8)..],
        if merchant.network == "mainnet" {
            "Mainnet"
        } else {
            "Sepolia"
        },
        explorer_base,
        tx.tx_hash,
    )
}

fn email_transport() -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let username =
        std::env::var("GMAIL_USER").context("GMAIL_USER must be set for email notifications")?;
    let password = std::env::var("GMAIL_APP_PASSWORD")
        .context("GMAIL_APP_PASSWORD must be set for email notifications")?;
    let creds = Credentials::new(username.clone(), password);

    Ok(
        AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")?
            .credentials(creds)
            .build(),
    )
}

pub async fn send_welcome(merchant: &Merchant) -> Result<()> {
    let from = Mailbox::new(
        Some("Zapcode".to_string()),
        std::env::var("GMAIL_USER")?.parse()?,
    );
    let to = Mailbox::new(None, merchant.email.parse()?);
    let html = build_welcome_html(&merchant.business_name, &merchant.wallet_address);

    let message = Message::builder()
        .from(from)
        .to(to)
        .subject(format!("Welcome to Zapcode — {}", merchant.business_name))
        .header(ContentType::TEXT_HTML)
        .body(html)?;

    let transport = email_transport()?;
    transport
        .send(message)
        .await
        .map_err(|err| anyhow!(err.to_string()))?;
    Ok(())
}

pub async fn send_payment_received(
    merchant: &Merchant,
    tx: &Transaction,
    fiat_amount: Option<f64>,
) -> Result<()> {
    let from = Mailbox::new(
        Some("Zapcode".to_string()),
        std::env::var("GMAIL_USER")?.parse()?,
    );
    let to = Mailbox::new(None, merchant.email.parse()?);
    let html = build_payment_html(merchant, tx, fiat_amount);

    let message = Message::builder()
        .from(from)
        .to(to)
        .subject(format!(
            "✓ {:.2} USDC received — {}",
            tx.amount, merchant.business_name
        ))
        .header(ContentType::TEXT_HTML)
        .body(html)?;

    let transport = email_transport()?;
    transport
        .send(message)
        .await
        .map_err(|err| anyhow!(err.to_string()))?;
    Ok(())
}
