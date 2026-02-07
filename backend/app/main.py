from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from .core.config import settings
from .api.prd_endpoints import router as prd_router
from .api.ralph_endpoints import router as ralph_router
from .api.websocket_endpoints import router as ws_router
from .api.project_endpoints import router as project_router
from .api.conversation_endpoints import router as conversation_router

app = FastAPI(
    title="Ralph Web UI API",
    description="Backend API for Ralph autonomous AI agent loop",
    version="1.0.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routers
app.include_router(prd_router)
app.include_router(ralph_router)
app.include_router(ws_router)
app.include_router(project_router)
app.include_router(conversation_router)


@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "message": "Ralph Web UI API",
        "version": "1.0.0",
        "docs": "/docs"
    }


@app.get("/health")
async def health():
    """Health check endpoint"""
    return {"status": "healthy"}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "app.main:app",
        host=settings.HOST,
        port=settings.PORT,
        reload=True
    )
