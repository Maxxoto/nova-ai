"""Telegram Bot channel implementation."""

import asyncio
import logging
from typing import Optional

from telegram import Update
from telegram.ext import (
    Application,
    CommandHandler,
    ContextTypes,
    MessageHandler,
    filters,
)

from app.infrastructure.bus.events import InboundMessage, OutboundMessage
from app.infrastructure.bus.queue import MessageBus
from app.infrastructure.channels.base import BaseChannel


logger = logging.getLogger(__name__)


class TelegramChannel(BaseChannel):
    """Telegram Bot channel."""

    name = "telegram"

    def __init__(
        self,
        token: str,
        bus: MessageBus,
        allow_list: Optional[list[str]] = None,
    ):
        """Initialize Telegram channel.

        Args:
            token: Bot token from @BotFather
            bus: Message bus for communication
            allow_list: Optional list of allowed user IDs
        """
        self.token = token
        self.bus = bus
        self.allow_list = allow_list
        self.application: Optional[Application] = None
        self._running = False

    async def start(self) -> None:
        """Start Telegram bot polling and outbound consumer."""
        try:
            # Build application
            self.application = Application.builder().token(self.token).build()

            # Add handlers
            self.application.add_handler(CommandHandler("start", self._handle_start))
            self.application.add_handler(CommandHandler("help", self._handle_help))
            self.application.add_handler(CommandHandler("new", self._handle_new))
            self.application.add_handler(
                MessageHandler(filters.TEXT & ~filters.COMMAND, self._handle_message)
            )

            # Initialize and start
            await self.application.initialize()
            await self.application.start()
            await self.application.updater.start_polling(drop_pending_updates=True)

            self._running = True
            logger.info("Telegram bot started")

            # Start outbound consumer in background
            asyncio.create_task(self._outbound_loop())

        except Exception as e:
            logger.error(f"Error starting Telegram bot: {e}")
            raise

    async def _outbound_loop(self) -> None:
        """Consume outbound messages and send to Telegram."""
        logger.info("Telegram outbound consumer started")
        while self._running:
            try:
                msg = await self.bus.consume_outbound()
                # Only handle messages for this channel
                if msg.channel == "telegram":
                    await self.send(msg)
            except Exception as e:
                logger.error(f"Error in Telegram outbound loop: {e}")
                await asyncio.sleep(1)

    async def stop(self) -> None:
        """Stop Telegram bot."""
        try:
            self._running = False

            if self.application:
                await self.application.updater.stop()
                await self.application.stop()
                await self.application.shutdown()

            logger.info("Telegram bot stopped")

        except Exception as e:
            logger.error(f"Error stopping Telegram bot: {e}")

    async def send(self, msg: OutboundMessage) -> None:
        """Send message via Telegram.

        Args:
            msg: Message to send
        """
        if not self.application:
            logger.error("Cannot send message: Telegram bot not initialized")
            return

        try:
            # Split long messages
            max_length = 4096
            content = msg.content

            while content:
                chunk = content[:max_length]
                content = content[max_length:]

                await self.application.bot.send_message(
                    chat_id=msg.chat_id,
                    text=chunk,
                    parse_mode=None,  # Plain text to avoid parsing errors
                )

            logger.debug(f"Sent message to Telegram chat {msg.chat_id}")

        except Exception as e:
            logger.error(f"Error sending Telegram message: {e}")

    async def _handle_start(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ) -> None:
        """Handle /start command."""
        user = update.effective_user

        if not self.is_allowed(str(user.id), self.allow_list):
            await update.message.reply_text(
                "Sorry, you're not authorized to use this bot."
            )
            return

        welcome_text = (
            f"👋 Hello {user.first_name}!\n\n"
            f"I'm Nova, your AI assistant.\n\n"
            f"Commands:\n"
            f"/new - Start new conversation\n"
            f"/help - Show help\n\n"
            f"Just send me a message to get started!"
        )

        await update.message.reply_text(welcome_text)

    async def _handle_help(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ) -> None:
        """Handle /help command."""
        help_text = (
            "🤖 Nova Bot Help\n\n"
            "Commands:\n"
            "/start - Start bot\n"
            "/new - Clear conversation history\n"
            "/help - Show this help\n\n"
            "I can:\n"
            "• Answer questions\n"
            "• Search the web (if configured)\n"
            "• Read/write files\n"
            "• Execute commands\n"
            "• Remember our conversations\n\n"
            "Just type your message!"
        )

        await update.message.reply_text(help_text)

    async def _handle_new(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ) -> None:
        """Handle /new command."""
        user = update.effective_user

        if not self.is_allowed(str(user.id), self.allow_list):
            return

        # Create system message to clear session
        msg = InboundMessage(
            channel="telegram",
            sender_id=str(user.id),
            chat_id=str(update.effective_chat.id),
            content="/new",
            metadata={"command": True},
        )

        await self.bus.publish_inbound(msg)
        await update.message.reply_text("✨ Started a new conversation!")

    async def _handle_message(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ) -> None:
        """Handle incoming text message."""
        user = update.effective_user

        if not self.is_allowed(str(user.id), self.allow_list):
            await update.message.reply_text(
                "Sorry, you're not authorized to use this bot."
            )
            return

        try:
            # Send "typing" indicator
            await context.bot.send_chat_action(
                chat_id=update.effective_chat.id,
                action="typing",
            )

            # Create inbound message
            msg = InboundMessage(
                channel="telegram",
                sender_id=str(user.id),
                chat_id=str(update.effective_chat.id),
                content=update.message.text,
                metadata={
                    "username": user.username,
                    "first_name": user.first_name,
                },
            )

            # Publish to bus
            await self.bus.publish_inbound(msg)
            logger.info(f"Received message from Telegram user {user.id}")

        except Exception as e:
            logger.error(f"Error handling Telegram message: {e}")
            await update.message.reply_text(
                "Sorry, I encountered an error processing your message."
            )

    @property
    def is_running(self) -> bool:
        """Check if channel is running."""
        return self._running
