# 🧠 AI Agent Memory Management Guide (`memory.md`)

This document defines the architecture, rules, and data formats for the Agent's memory system. It serves as the primary specification for the **Planner** and **Memory Nodes** of the agent pipeline.

---

## 1. Memory Architecture Overview

The agent utilizes a **Hybrid Memory System** composed of two main types, managed by a Retrieval-Augmented Generation (RAG) framework:

| Memory Type | Storage Mechanism | Purpose | Management Strategy |
| :--- | :--- | :--- | :--- |
| **Short-Term (STM)** | LLM Context Window (Prompt) | Immediate conversational flow and intent. | **Context Trimming** and **Summarization**. |
| **Long-Term (LTM)** | Vector Database (Pinecone) | Persistence of key facts, preferences, and long-term plans. | **Information Extraction** and **Semantic Retrieval** (RAG). |

---

## 2. Long-Term Memory (LTM) Storage Rules

The **Memory Write Node** is responsible for converting unstructured conversation into structured, atomic facts for storage in the Pinecone Vector Database. This is the **Information Extraction** step.

### 2.1. Fact Structure (The Atomic Knowledge Triple)

Each memory entry **must** be formatted as a single, atomic natural language sentence. This format optimizes the vector embedding for semantic search.

| Field | Purpose in Memory System | Example Content |
| :--- | :--- | :--- |
| **Fact Type** | A clear category tag for filtering and retrieval. | `USER_PREFERENCE:`, `USER_INFO:`, `CONFIRMED_PLAN:` |
| **Content (Vectorized)**| The complete, minimal, and self-contained fact sentence. | `The user's favorite color is blue.` |
| **Metadata Tag** | Identifiers stored alongside the vector for filtering. | `user_id: "alex_123"`, `timestamp: 2025-11-03`, `session_id: "s4f7h9"` |

### 2.2. Write Triggers (When to Save to LTM)

The **Memory Write Node** should be activated and run the Information Extraction process under these conditions:

1.  **Fact Confirmation:** When the user explicitly **confirms** a piece of information.
2.  **Explicit Intent/Preference:** When the user **states a critical, enduring preference or intent**.
3.  **Plan Agreement:** When the agent and user **agree on a multi-step plan or objective**.

---

## 3. Memory Retrieval and Integration (RAG)

The **Memory Recall Router** decides which memories to retrieve and how to present them to the **Planner / Prompt Builder**.

### 3.1. Retrieval Triggers

The agent must search LTM before generating a response if the **Intent Detector** identifies any of the following:

1.  **Query Type:** The user asks a question that requires historical or factual context.
2.  **Entity Mention:** The user mentions an entity (a project, plan, or preference) stored in LTM.
3.  **Ambiguity:** The user's query is highly ambiguous, requiring LTM search to contextualize.

### 3.2. Prompt Integration Format

Retrieved facts **must** be injected into the Planner's final prompt using a dedicated XML tag structure for the LLM:

```xml
<RECALLED_MEMORY>
  - [Fact Type]: [Content]
  - [Fact Type]: [Content]
</RECALLED_MEMORY>
