#!/usr/bin/env python3
"""
Production-grade implementation for a brain-data driven autonomous agent.
This module includes:
    - EEG/brain data preprocessing
    - Feature extraction (e.g. spectrogram transformation)
    - Representation learning using an autoencoder-inspired architecture
      (combining CNNs and RNNs to capture spatial and temporal patterns)
    - An RL decision module that maps latent engrams to actions

Author: Your Name
Date: 2025-02-21
"""

from typing import Tuple, Optional
import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
from loguru import logger
import matplotlib.pyplot as plt


# =============================================================================
# Data Preprocessing and Feature Extraction Modules
# =============================================================================

class EEGPreprocessor:
    """
    Preprocess raw EEG/brain data.
    
    This class applies filtering, artifact removal, and normalization to raw data.
    """
    def __init__(self, sample_rate: int = 256) -> None:
        """
        Args:
            sample_rate (int): Sampling rate of the EEG data.
        """
        self.sample_rate = sample_rate
        logger.info("EEGPreprocessor initialized with sample_rate={}", sample_rate)

    def preprocess(self, data: np.ndarray) -> torch.Tensor:
        """
        Preprocess raw EEG data.

        Args:
            data (np.ndarray): Raw EEG data of shape (channels, time).

        Returns:
            torch.Tensor: Preprocessed EEG data.
        """
        logger.debug("Starting EEG data preprocessing...")
        try:
            # Remove DC offset and normalize
            data = data - np.mean(data, axis=1, keepdims=True)
            data = data / (np.std(data, axis=1, keepdims=True) + 1e-6)
            tensor_data = torch.tensor(data, dtype=torch.float32)
            logger.debug("EEG preprocessing complete. Shape: {}", tensor_data.shape)
            return tensor_data
        except Exception as e:
            logger.error("Error in preprocessing: {}", e)
            raise


class FeatureExtractor:
    """
    Extract features from preprocessed EEG data.
    
    This module converts time-domain signals into spectrograms or other 
    spatial-temporal representations.
    """
    def __init__(self, n_fft: int = 64, hop_length: int = 16) -> None:
        """
        Args:
            n_fft (int): Number of FFT components.
            hop_length (int): Hop length for FFT windows.
        """
        self.n_fft = n_fft
        self.hop_length = hop_length
        logger.info("FeatureExtractor initialized with n_fft={}, hop_length={}", n_fft, hop_length)

    def extract_features(self, eeg_tensor: torch.Tensor) -> torch.Tensor:
        """
        Compute a simple spectrogram for each channel.

        Args:
            eeg_tensor (torch.Tensor): Preprocessed EEG data (channels, time).

        Returns:
            torch.Tensor: Feature tensor of shape (channels, freq_bins, time_frames).
        """
        logger.debug("Extracting features from EEG tensor with shape: {}", eeg_tensor.shape)
        try:
            # Use torch.stft to compute the short-time Fourier transform per channel.
            spectrograms = []
            for channel in eeg_tensor:
                # torch.stft returns a complex tensor; we take the magnitude.
                spec = torch.stft(channel, n_fft=self.n_fft, hop_length=self.hop_length, return_complex=True)
                spectrograms.append(torch.abs(spec))
            features = torch.stack(spectrograms, dim=0)
            logger.debug("Feature extraction complete. Output shape: {}", features.shape)
            return features
        except Exception as e:
            logger.error("Error during feature extraction: {}", e)
            raise


# =============================================================================
# Representation Learning: The Engram Encoder
# =============================================================================

class EngramEncoder(nn.Module):
    """
    Encoder module to learn a latent representation (engram) from EEG features.
    
    Combines convolutional layers for spatial feature extraction and GRU layers
    for temporal dynamics.
    """
    def __init__(self, in_channels: int, latent_dim: int, hidden_dim: int = 128) -> None:
        """
        Args:
            in_channels (int): Number of input channels (e.g., EEG channels).
            latent_dim (int): Dimensionality of the latent representation.
            hidden_dim (int): Hidden dimension for GRU.
        """
        super(EngramEncoder, self).__init__()
        logger.info("Initializing EngramEncoder with in_channels={}, latent_dim={}", in_channels, latent_dim)
        
        # Convolutional block for spatial feature extraction
        self.conv_block = nn.Sequential(
            nn.Conv2d(in_channels=in_channels, out_channels=16, kernel_size=3, padding=1),
            nn.BatchNorm2d(16),
            nn.ReLU(),
            nn.MaxPool2d(2)
        )
        
        # For this demo, we assume the input feature maps have freq_bins=64 and time_frames=256,
        # and after pooling become 32 and 128 respectively.
        # GRU for temporal sequence modeling (flatten spatial dimensions)
        self.gru = nn.GRU(input_size=16 * 32, hidden_size=hidden_dim, batch_first=True)
        self.fc = nn.Linear(hidden_dim, latent_dim)
    
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Forward pass through the encoder.

        Args:
            x (torch.Tensor): Input feature tensor of shape (batch, channels, freq_bins, time_frames).

        Returns:
            torch.Tensor: Latent representation of shape (batch, latent_dim).
        """
        logger.debug("EngramEncoder forward pass with input shape: {}", x.shape)
        # Apply convolutional block
        conv_out = self.conv_block(x)
        logger.debug("After conv block, shape: {}", conv_out.shape)
        batch_size, channels, freq_bins, time_frames = conv_out.shape
        
        # Flatten spatial dimensions except the temporal dimension
        conv_out = conv_out.view(batch_size, channels * freq_bins, time_frames).permute(0, 2, 1)
        
        # Process temporal dynamics with GRU
        gru_out, _ = self.gru(conv_out)
        # Use last hidden state as summary
        gru_last = gru_out[:, -1, :]
        latent = self.fc(gru_last)
        logger.debug("Latent representation shape: {}", latent.shape)
        return latent

    def encode(self, x: torch.Tensor) -> torch.Tensor:
        """
        Convenience method for encoding.

        Args:
            x (torch.Tensor): Input tensor.

        Returns:
            torch.Tensor: Latent representation.
        """
        return self.forward(x)


# =============================================================================
# RL-based Decision Module
# =============================================================================

class RLAgent(nn.Module):
    """
    Reinforcement Learning agent that maps latent engrams to actions.
    """
    def __init__(self, latent_dim: int, num_actions: int, hidden_dim: int = 64) -> None:
        """
        Args:
            latent_dim (int): Dimensionality of the latent representation.
            num_actions (int): Number of possible actions.
            hidden_dim (int): Hidden layer dimension.
        """
        super(RLAgent, self).__init__()
        logger.info("Initializing RLAgent with latent_dim={}, num_actions={}", latent_dim, num_actions)
        self.policy_net = nn.Sequential(
            nn.Linear(latent_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, num_actions)
        )
    
    def forward(self, latent: torch.Tensor) -> torch.Tensor:
        """
        Compute action logits from latent representation.

        Args:
            latent (torch.Tensor): Latent vector (batch, latent_dim).

        Returns:
            torch.Tensor: Action logits (batch, num_actions).
        """
        logits = self.policy_net(latent)
        logger.debug("RLAgent forward pass logits shape: {}", logits.shape)
        return logits


# =============================================================================
# Dummy Dataset for Demonstration
# =============================================================================

class DummyEEGDataset(Dataset):
    """
    A dummy dataset for EEG data demonstration purposes.
    """
    def __init__(self, num_samples: int = 100, channels: int = 8, time_steps: int = 256) -> None:
        """
        Args:
            num_samples (int): Number of samples in the dataset.
            channels (int): Number of EEG channels.
            time_steps (int): Number of time steps per sample.
        """
        super().__init__()
        self.num_samples = num_samples
        self.channels = channels
        self.time_steps = time_steps
        # Random data simulating raw EEG signals as float64; conversion happens in __getitem__.
        self.data = np.random.randn(num_samples, channels, time_steps)
        logger.info("DummyEEGDataset created with {} samples", num_samples)

    def __len__(self) -> int:
        return self.num_samples

    def __getitem__(self, idx: int) -> torch.Tensor:
        # Convert to torch.Tensor with float32 to ensure consistency with model weights.
        return torch.tensor(self.data[idx], dtype=torch.float32)


# =============================================================================
# Training and Integration
# =============================================================================

def train_representation(
    encoder: EngramEncoder,
    dataloader: DataLoader,
    num_epochs: int = 10,
    lr: float = 1e-3,
    device: Optional[torch.device] = None
) -> None:
    """
    Train the representation learning model.
    
    This function demonstrates a dummy training loop for the engram encoder.
    
    Args:
        encoder (EngramEncoder): The encoder model.
        dataloader (DataLoader): DataLoader providing training data.
        num_epochs (int): Number of training epochs.
        lr (float): Learning rate.
        device (Optional[torch.device]): Device to run training on.
    """
    device = device or torch.device("cuda" if torch.cuda.is_available() else "cpu")
    encoder.to(device)
    optimizer = optim.Adam(encoder.parameters(), lr=lr)
    criterion = nn.MSELoss()  # Dummy loss for an autoencoding-like task

    encoder.train()
    for epoch in range(num_epochs):
        epoch_loss = 0.0
        for batch_idx, batch_data in enumerate(dataloader):
            batch_data = batch_data.to(device)  # shape: (batch, channels, time)
            # Simulate feature extraction: add a frequency dimension and repeat to get a realistic size.
            features = batch_data.unsqueeze(2).repeat(1, 1, 64, 1)  # (batch, channels, 64, time)
            
            optimizer.zero_grad()
            latent = encoder(features)
            # Dummy reconstruction loss: using the latent vector as its own target.
            loss = criterion(latent, latent.detach())
            loss.backward()
            optimizer.step()
            epoch_loss += loss.item()
        
        logger.info("Epoch [{}/{}] Loss: {:.4f}", epoch+1, num_epochs, epoch_loss / len(dataloader))


def train_rl_agent(
    agent: RLAgent,
    encoder: EngramEncoder,
    dataloader: DataLoader,
    num_epochs: int = 10,
    lr: float = 1e-3,
    device: Optional[torch.device] = None
) -> None:
    """
    Train the RL agent to map latent engrams to actions.

    This dummy training loop demonstrates how the RL agent might be trained
    given latent representations from the encoder.

    Args:
        agent (RLAgent): The reinforcement learning agent.
        encoder (EngramEncoder): The encoder that produces latent representations.
        dataloader (DataLoader): DataLoader providing training data.
        num_epochs (int): Number of training epochs.
        lr (float): Learning rate.
        device (Optional[torch.device]): Device to run training on.
    """
    device = device or torch.device("cuda" if torch.cuda.is_available() else "cpu")
    agent.to(device)
    encoder.to(device)
    
    optimizer = optim.Adam(agent.parameters(), lr=lr)
    criterion = nn.CrossEntropyLoss()  # Dummy target classification task
    agent.train()
    encoder.eval()  # Freeze encoder during RL training (if desired)

    for epoch in range(num_epochs):
        epoch_loss = 0.0
        for batch_idx, batch_data in enumerate(dataloader):
            batch_data = batch_data.to(device)
            # Simulate feature extraction as before.
            features = batch_data.unsqueeze(2).repeat(1, 1, 64, 1)  # (batch, channels, 64, time)
            with torch.no_grad():
                latent = encoder(features)
            # Dummy target: random actions for each sample in the batch.
            targets = torch.randint(0, agent.policy_net[-1].out_features, (latent.size(0),), device=device)
            
            optimizer.zero_grad()
            logits = agent(latent)
            loss = criterion(logits, targets)
            loss.backward()
            optimizer.step()
            epoch_loss += loss.item()

        logger.info("RL Epoch [{}/{}] Loss: {:.4f}", epoch+1, num_epochs, epoch_loss / len(dataloader))


def main() -> None:
    """
    Main entry point for training the engram encoder and RL agent.
    """
    # Set up device, logging, and seed for reproducibility.
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    torch.manual_seed(42)
    
    # Create dummy dataset and dataloader
    dataset = DummyEEGDataset(num_samples=200, channels=8, time_steps=256)
    dataloader = DataLoader(dataset, batch_size=16, shuffle=True, num_workers=2)

    # Instantiate modules
    encoder = EngramEncoder(in_channels=8, latent_dim=32, hidden_dim=128)
    agent = RLAgent(latent_dim=32, num_actions=4, hidden_dim=64)

    logger.info("Starting representation training...")
    train_representation(encoder, dataloader, num_epochs=5, lr=1e-3, device=device)

    logger.info("Starting RL agent training...")
    train_rl_agent(agent, encoder, dataloader, num_epochs=5, lr=1e-3, device=device)

    logger.info("Training complete.")

    # Optionally, save the models
    torch.save(encoder.state_dict(), "engram_encoder.pth")
    torch.save(agent.state_dict(), "rl_agent.pth")
    logger.info("Models saved.")
import torch
from loguru import logger

def run_inference(encoder: EngramEncoder, agent: RLAgent, raw_eeg: torch.Tensor, device: torch.device) -> int:
    """
    Runs inference on a single raw EEG sample.

    Args:
        encoder (EngramEncoder): Trained encoder model.
        agent (RLAgent): Trained RL agent.
        raw_eeg (torch.Tensor): Raw EEG data tensor of shape (channels, time).
        device (torch.device): Device for computation.

    Returns:
        int: The selected action index.
    """
    # Ensure the model is in evaluation mode.
    encoder.eval()
    agent.eval()

    # Preprocess and simulate feature extraction:
    # Assuming raw_eeg is already preprocessed; if not, you should preprocess it.
    # Add a batch dimension and simulate spectrogram feature extraction.
    # For instance, raw_eeg shape: (channels, time) --> (1, channels, 64, time)
    features = raw_eeg.unsqueeze(0).unsqueeze(2).repeat(1, 1, 64, 1).to(device)

    with torch.no_grad():
        # Get latent representation from the encoder.
        latent = encoder(features)
        # Pass latent vector through the RL agent to obtain action logits.
        logits = agent(latent)
        # Convert logits to probabilities using softmax.
        probabilities = torch.softmax(logits, dim=-1)
        # Choose action with the highest probability.
        action = torch.argmax(probabilities, dim=-1).item()

    logger.info("Selected action: {}", action)
    return action

# Example usage:
if __name__ == "__main__":
    main()
    # Assume that encoder and agent have been previously trained and loaded.
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    # Load pre-trained models (or use the ones you just trained)
    encoder = EngramEncoder(in_channels=8, latent_dim=32, hidden_dim=128).to(device)
    agent = RLAgent(latent_dim=32, num_actions=4, hidden_dim=64).to(device)
    encoder.load_state_dict(torch.load("engram_encoder.pth", map_location=device))
    agent.load_state_dict(torch.load("rl_agent.pth", map_location=device))
    
    # Create or obtain a raw EEG sample. For demonstration, we create dummy data.
    # Suppose we have 8 channels and 256 time steps.
    dummy_eeg = torch.randn(8, 256, dtype=torch.float32)
    
    # Run inference to get the selected action.
    selected_action = run_inference(encoder, agent, dummy_eeg, device)
