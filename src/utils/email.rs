use crate::errors::AppError;
use resend_rs::{Resend, types::CreateEmailBaseOptions};

/// Send OTP email via Resend
pub async fn send_otp_email(
    to_email: &str,
    otp_code: &str,
    resend_api_key: &str,
) -> Result<(), AppError> {
    let resend = Resend::new(resend_api_key);

    let from = "Spot Feed <onboarding@resend.dev>";
    let to = [to_email];
    let subject = "Your Spot Feed Verification Code";

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
                .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; text-align: center; border-radius: 10px 10px 0 0; }}
                .content {{ background: #f9f9f9; padding: 30px; border-radius: 0 0 10px 10px; }}
                .otp-code {{ font-size: 32px; font-weight: bold; color: #667eea; text-align: center; letter-spacing: 8px; margin: 20px 0; padding: 15px; background: white; border-radius: 8px; }}
                .footer {{ text-align: center; margin-top: 20px; color: #666; font-size: 12px; }}
            </style>
        </head>
        <body>
            <div class="container">
                <div class="header">
                    <h1>🎯 Spot Feed</h1>
                    <p>Location-Based Social Networking</p>
                </div>
                <div class="content">
                    <h2>Verify Your Email</h2>
                    <p>Thanks for signing up! Use the code below to verify your email address:</p>
                    <div class="otp-code">{}</div>
                    <p>This code will expire in <strong>10 minutes</strong>.</p>
                    <p>If you didn't request this code, please ignore this email.</p>
                </div>
                <div class="footer">
                    <p>© 2025 Spot Feed. All rights reserved.</p>
                </div>
            </div>
        </body>
        </html>
        "#,
        otp_code
    );

    let email = CreateEmailBaseOptions::new(from, to, subject).with_html(&html);

    resend.emails.send(email).await.map_err(|e| {
        tracing::error!("Failed to send OTP email: {:?}", e);
        AppError::InternalError("Failed to send verification email".to_string())
    })?;

    tracing::info!("OTP email sent successfully to {}", to_email);
    Ok(())
}
