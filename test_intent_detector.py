#!/usr/bin/env python3
"""Test script for IntentDetector functionality."""

import asyncio
import sys
import os

# Add the src directory to Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'src'))

from domain.entities.agent_state import AgentState
from langchain_core.messages import HumanMessage, AIMessage
from infrastructure.adapters.llm_providers.groq import GroqLLMClient
from application.services.nodes.intent_detector import IntentDetector


async def test_intent_detector():
    """Test the intent detector with sample messages."""
    
    # Initialize LLM client
    llm_client = GroqLLMClient()
    
    # Initialize intent detector
    intent_detector = IntentDetector(llm_client)
    
    # Test messages for different intents
    test_cases = [
        {
            "message": "Can you help me create a step-by-step plan for launching a new product?",
            "expected_intent": "planning"
        },
        {
            "message": "What's the weather like today?",
            "expected_intent": "general_chat"
        },
        {
            "message": "I need a strategy for improving customer retention",
            "expected_intent": "planning"
        },
        {
            "message": "Hello, how are you?",
            "expected_intent": "general_chat"
        },
        {
            "message": "Can you break down the process of building a web application?",
            "expected_intent": "planning"
        }
    ]
    
    print("Testing IntentDetector with sample messages...")
    print("=" * 60)
    
    for i, test_case in enumerate(test_cases, 1):
        print(f"\nTest Case {i}:")
        print(f"Message: '{test_case['message']}'")
        print(f"Expected intent: {test_case['expected_intent']}")
        
        # Create agent state with the test message
        state = AgentState(
            messages=[HumanMessage(content=test_case['message'])]
        )
        
        try:
            # Detect intent
            result_state = await intent_detector.detect_intent(state)
            
            print(f"Detected intent: {result_state.intent}")
            print(f"Needs planning: {result_state.needs_planning}")
            
            # Check if detection matches expectation
            if result_state.intent == test_case['expected_intent']:
                print("✅ PASS")
            else:
                print("❌ FAIL - Intent mismatch")
                
        except Exception as e:
            print(f"❌ ERROR: {str(e)}")
    
    print("\n" + "=" * 60)
    print("Intent detector test completed.")


if __name__ == "__main__":
    asyncio.run(test_intent_detector())