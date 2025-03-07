import os
import json
import torch
from torch import Tensor
from loguru import logger
from typing import Optional, List
from PIL import Image
from datetime import datetime
import threading
import time

# Pretrained Dialogue Generation Imports
from transformers import GPT2LMHeadModel, GPT2Tokenizer

# Pretrained Asset Generation Imports
from diffusers import StableDiffusionPipeline
from torchvision import transforms

# --------------------------
# CONFIGURATION & GLOBALS
# --------------------------

DEVICE = "cuda" if torch.cuda.is_available() else "cpu"
ASSET_CACHE_DIR = "./asset_cache"
STATE_SAVE_DIR = "./game_states"

os.makedirs(ASSET_CACHE_DIR, exist_ok=True)
os.makedirs(STATE_SAVE_DIR, exist_ok=True)


# --------------------------
# UTILITY FUNCTIONS
# --------------------------

def safe_write_json(filepath: str, data: dict) -> None:
    try:
        with open(filepath, "w") as fp:
            json.dump(data, fp, indent=2)
        logger.info("Game state saved to {}", filepath)
    except Exception as e:
        logger.exception("Failed to save game state to {}: {}", filepath, e)
        raise


def generate_state_filename(prefix: str = "game_state") -> str:
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    return os.path.join(STATE_SAVE_DIR, f"{prefix}_{timestamp}.json")


# --------------------------
# SCENE PARSING
# --------------------------

class SceneParser:
    def __init__(self) -> None:
        logger.info("Initializing SceneParser.")

    def parse_text(self, prompt: str) -> dict:
        logger.debug("Parsing text prompt: {}", prompt)
        try:
            # Dummy parser: In production, use an NLP model to extract details.
            scene = {
                "environment": "fantasy_landscape",
                "description": prompt,
                "objects": [
                    {"type": "tree", "position": (10, 20), "state": "static"},
                    {"type": "rock", "position": (15, 25), "state": "static"}
                ],
                "characters": [
                    {"name": "hero", "position": (5, 5), "mood": "determined"},
                    {"name": "villager", "position": (12, 18), "mood": "curious"}
                ],
                "events": []
            }
            logger.info("Scene parsed successfully.")
            return scene
        except Exception as e:
            logger.exception("Error parsing scene: {}", e)
            raise

    def update_scene(self, scene: dict, updates: dict) -> dict:
        logger.debug("Updating scene with {}", updates)
        try:
            scene.update(updates)
            logger.info("Scene updated.")
            return scene
        except Exception as e:
            logger.exception("Failed to update scene: {}", e)
            raise


# --------------------------
# ASSET GENERATION & CACHING
# --------------------------

class AssetCache:
    def __init__(self, cache_dir: str = ASSET_CACHE_DIR) -> None:
        logger.info("Initializing AssetCache in {}", cache_dir)
        self.cache_dir = cache_dir

    def get_cache_path(self, prompt: str) -> str:
        safe_prompt = prompt.replace(" ", "_").replace(",", "").lower()
        filename = f"{safe_prompt}.pt"
        return os.path.join(self.cache_dir, filename)

    def load(self, prompt: str) -> Optional[Tensor]:
        path = self.get_cache_path(prompt)
        if os.path.exists(path):
            try:
                asset = torch.load(path, map_location=DEVICE)
                logger.info("Loaded cached asset for prompt: {}", prompt)
                return asset
            except Exception as e:
                logger.exception("Failed to load cached asset from {}: {}", path, e)
                return None
        return None

    def save(self, prompt: str, asset: Tensor) -> None:
        path = self.get_cache_path(prompt)
        try:
            torch.save(asset, path)
            logger.info("Saved asset cache for prompt: {} to {}", prompt, path)
        except Exception as e:
            logger.exception("Failed to save asset cache: {}", e)
            raise


class AssetGenerator:
    def __init__(self, device: str = DEVICE) -> None:
        logger.info("Loading Stable Diffusion pipeline on {}.", device)
        try:
            self.pipe = StableDiffusionPipeline.from_pretrained(
                "CompVis/stable-diffusion-v1-4", revision="fp16", torch_dtype=torch.float16
            )
            self.pipe = self.pipe.to(device)
            self.device = device
            self.to_tensor = transforms.Compose([
                transforms.ToTensor(),
            ])
            self.cache = AssetCache()
            logger.info("Stable Diffusion pipeline loaded successfully.")
        except Exception as e:
            logger.exception("Error loading Stable Diffusion pipeline: {}", e)
            raise

    def generate(self, prompt: str, num_inference_steps: int = 50, guidance_scale: float = 7.5) -> Tensor:
        logger.info("Generating asset for prompt: {}", prompt)
        try:
            # Check cache first.
            cached_asset = self.cache.load(prompt)
            if cached_asset is not None:
                return cached_asset

            output = self.pipe(prompt, num_inference_steps=num_inference_steps, guidance_scale=guidance_scale)
            image: Image.Image = output.images[0]
            logger.debug("Asset image generated, converting to tensor.")
            tensor = self.to_tensor(image).unsqueeze(0)  # Shape: (B, C, H, W)
            if tensor.ndim != 4:
                raise ValueError("AssetGenerator output is not 4D.")
            self.cache.save(prompt, tensor)
            logger.info("Asset generation complete, tensor shape: {}", tensor.shape)
            return tensor.to(self.device)
        except Exception as e:
            logger.exception("Error during asset generation: {}", e)
            raise


# --------------------------
# DIALOGUE GENERATION
# --------------------------

class DialogueGenerator:
    def __init__(self, device: str = DEVICE) -> None:
        logger.info("Loading GPT-2 model on {}.", device)
        try:
            self.tokenizer = GPT2Tokenizer.from_pretrained("gpt2")
            self.model = GPT2LMHeadModel.from_pretrained("gpt2").to(device)
            self.device = device
            logger.info("GPT-2 loaded successfully.")
        except Exception as e:
            logger.exception("Error loading GPT-2 model: {}", e)
            raise

    def generate_dialogue(self, context: str, max_length: int = 50) -> str:
        logger.info("Generating dialogue for context: {}", context)
        try:
            inputs = self.tokenizer.encode(context, return_tensors="pt").to(self.device)
            outputs = self.model.generate(
                inputs,
                max_length=max_length,
                num_return_sequences=1,
                do_sample=True,
                top_p=0.95,
                top_k=50
            )
            dialogue = self.tokenizer.decode(outputs[0], skip_special_tokens=True)
            logger.info("Dialogue generation complete.")
            return dialogue
        except Exception as e:
            logger.exception("Error generating dialogue: {}", e)
            raise


# --------------------------
# GAME ENGINE & EVENT MANAGEMENT
# --------------------------

class GameEngine:
    def __init__(self, simulation_steps: int = 10) -> None:
        logger.info("Initializing GameEngine with {} simulation steps.", simulation_steps)
        self.simulation_steps = simulation_steps
        self.event_manager = EventManager()

    def simulate(self, scene: dict, asset_tensor: Tensor) -> dict:
        logger.debug("Starting simulation for scene: {}", scene.get("environment"))
        try:
            for step in range(self.simulation_steps):
                logger.trace("Simulation step {}/{}", step + 1, self.simulation_steps)
                scene = self.event_manager.process_events(scene, step)
                time.sleep(0.05)  # simulate time delay per step
            scene["simulation"] = "completed"
            scene["asset_tensor"] = asset_tensor.detach().cpu().numpy().tolist()
            logger.info("Simulation completed successfully.")
            return scene
        except Exception as e:
            logger.exception("Error during simulation: {}", e)
            raise


class EventManager:
    def __init__(self) -> None:
        logger.info("Initializing EventManager.")
        self.events: List[dict] = []

    def add_event(self, event: dict) -> None:
        logger.debug("Adding event: {}", event)
        self.events.append(event)

    def process_events(self, scene: dict, step: int) -> dict:
        logger.debug("Processing events at simulation step {}", step)
        try:
            # Process scheduled events; in production, this could handle collisions, state changes, etc.
            for event in self.events:
                if event.get("trigger_step") == step:
                    logger.info("Triggering event: {}", event)
                    scene = self.apply_event(scene, event)
            # Optionally clear events that have been processed.
            self.events = [e for e in self.events if e.get("trigger_step", -1) > step]
            return scene
        except Exception as e:
            logger.exception("Error processing events: {}", e)
            raise

    def apply_event(self, scene: dict, event: dict) -> dict:
        logger.debug("Applying event: {}", event)
        try:
            # Dummy event application: update object state or character mood.
            target = event.get("target", "object")
            if target == "object" and scene.get("objects"):
                scene["objects"][0]["state"] = event.get("new_state", "changed")
            elif target == "character" and scene.get("characters"):
                scene["characters"][0]["mood"] = event.get("new_mood", "excited")
            scene.setdefault("events", []).append(event)
            logger.info("Event applied successfully.")
            return scene
        except Exception as e:
            logger.exception("Error applying event: {}", e)
            raise


# --------------------------
# AGENT CONTROLLER & SCHEDULER
# --------------------------

class AgentController:
    def __init__(self) -> None:
        logger.info("Initializing AgentController.")
        self.scheduler = AgentScheduler()

    def update_agents(self, scene: dict, dialogue: str) -> dict:
        logger.debug("Updating agents with dialogue: {}", dialogue)
        try:
            for agent in scene.get("characters", []):
                # Schedule a behavior update based on dialogue.
                agent["last_dialogue"] = dialogue
                agent["next_action"] = self.scheduler.schedule_action(agent)
            logger.info("Agent behaviors updated successfully.")
            return scene
        except Exception as e:
            logger.exception("Error updating agents: {}", e)
            raise


class AgentScheduler:
    def __init__(self) -> None:
        logger.info("Initializing AgentScheduler.")

    def schedule_action(self, agent: dict) -> str:
        try:
            # Dummy scheduling: randomly choose an action based on the agent's mood.
            mood = agent.get("mood", "neutral")
            if mood == "determined":
                action = "advance"
            elif mood == "curious":
                action = "explore"
            else:
                action = "idle"
            logger.debug("Scheduled action '{}' for agent with mood '{}'", action, mood)
            return action
        except Exception as e:
            logger.exception("Error scheduling action for agent: {}", e)
            raise


# --------------------------
# GAME STATE SAVER
# --------------------------

class GameStateSaver:
    def __init__(self) -> None:
        logger.info("Initializing GameStateSaver.")

    def save_state(self, state: dict, filename_prefix: str = "game_state") -> None:
        try:
            filepath = generate_state_filename(filename_prefix)
            safe_write_json(filepath, state)
        except Exception as e:
            logger.exception("Failed to save game state: {}", e)
            raise


# --------------------------
# FULL STACK GAME MODEL
# --------------------------

class FullStackGameModel:
    def __init__(self) -> None:
        logger.info("Initializing FullStackGameModel pipeline.")
        self.scene_parser = SceneParser()
        self.asset_generator = AssetGenerator(device=DEVICE)
        self.dialogue_generator = DialogueGenerator(device=DEVICE)
        self.game_engine = GameEngine(simulation_steps=20)
        self.agent_controller = AgentController()
        self.state_saver = GameStateSaver()

    def forward(self, prompt: str) -> dict:
        logger.info("Executing full pipeline for prompt.")
        try:
            scene = self.scene_parser.parse_text(prompt)
            # Dynamically schedule an event during simulation.
            scene["events"].append({
                "trigger_step": 5,
                "target": "character",
                "new_mood": "excited",
                "description": "A sudden burst of inspiration."
            })
            asset_prompt = f"{scene.get('environment')} scene with detailed textures and dynamic lighting"
            asset_tensor = self.asset_generator.generate(asset_prompt)
            dialogue_context = f"{scene.get('environment')} with {len(scene.get('characters', []))} characters"
            dialogue = self.dialogue_generator.generate_dialogue(dialogue_context)
            simulated_scene = self.game_engine.simulate(scene, asset_tensor)
            final_scene = self.agent_controller.update_agents(simulated_scene, dialogue)
            # Persist game state
            self.state_saver.save_state(final_scene)
            logger.info("Full pipeline executed successfully.")
            return final_scene
        except Exception as e:
            logger.exception("Error in full pipeline execution: {}", e)
            raise

    def run_background_simulation(self, prompt: str, interval: float = 1.0) -> None:
        """
        Run the simulation repeatedly in a background thread to emulate dynamic game world updates.
        """
        def simulation_loop():
            while True:
                try:
                    state = self.forward(prompt)
                    logger.info("Background simulation update complete. State ID: {}", state.get("simulation"))
                except Exception as e:
                    logger.exception("Background simulation error: {}", e)
                time.sleep(interval)

        thread = threading.Thread(target=simulation_loop, daemon=True)
        thread.start()
        logger.info("Background simulation started.")


# --------------------------
# PIPELINE RUNNER (IMPORTABLE, NO CLI)
# --------------------------

def run_pipeline(prompt: str) -> dict:
    """
    Runs the full game pipeline for a given prompt and returns the final state.
    This function can be imported and called by external modules.
    """
    try:
        model = FullStackGameModel()
        final_state = model.forward(prompt)
        return final_state
    except Exception as e:
        logger.exception("Error running pipeline: {}", e)
        raise


# --------------------------
# EXAMPLE USAGE (COMMENTED OUT)
# --------------------------
#
# The following example usage code is commented out to comply with the "no CLI" requirement.
#
if __name__ == "__main__":
    prompt = "A mystical forest with ancient trees and wandering spirits."
    state = run_pipeline(prompt)
    logger.info("Final game state: {}", state)
