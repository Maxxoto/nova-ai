import re

from langchain_core.output_parsers import BaseOutputParser


class ThinkCleanerParser(BaseOutputParser):
    """
    Output parser to remove <think> tokens and their content from the LLM output.
    """

    def parse(self, text: str) -> str:
        return re.sub(r"<think>.*?</think>", "", text, flags=re.DOTALL).strip()

    @property
    def _lc_name(self) -> str:
        return "ThinkCleanerParser"
