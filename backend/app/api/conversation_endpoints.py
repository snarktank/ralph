from fastapi import APIRouter, HTTPException
from typing import List
from ..models.conversation import Conversation
from ..services.conversation_manager import conversation_manager

router = APIRouter(prefix="/api/conversations", tags=["conversations"])


@router.get("/orchestrator", response_model=Conversation)
async def get_orchestrator_conversation():
    """Get the full orchestrator conversation history"""
    return conversation_manager.get_orchestrator_conversation()


@router.get("/subagents", response_model=List[Conversation])
async def get_all_subagent_conversations():
    """Get all subagent conversations"""
    return conversation_manager.get_all_subagent_conversations()


@router.get("/subagents/{iteration}", response_model=Conversation)
async def get_subagent_conversation(iteration: int):
    """Get a specific subagent conversation by iteration"""
    conversation = conversation_manager.get_subagent_conversation(iteration)
    if not conversation:
        raise HTTPException(status_code=404, detail=f"No conversation found for iteration {iteration}")
    return conversation


@router.get("/summary")
async def get_conversation_summary():
    """Get summary of all conversations"""
    return conversation_manager.get_conversation_summary()


@router.delete("/")
async def clear_all_conversations():
    """Clear all conversation history"""
    conversation_manager.clear_all_conversations()
    return {"message": "All conversations cleared"}


@router.delete("/orchestrator")
async def clear_orchestrator_conversation():
    """Clear orchestrator conversation"""
    conversation_manager.clear_orchestrator_conversation()
    return {"message": "Orchestrator conversation cleared"}


@router.delete("/subagents")
async def clear_subagent_conversations():
    """Clear all subagent conversations"""
    conversation_manager.clear_subagent_conversations()
    return {"message": "Subagent conversations cleared"}
