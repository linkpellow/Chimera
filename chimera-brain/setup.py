from setuptools import setup, find_packages

setup(
    name="chimera-brain",
    version="0.1.0",
    packages=find_packages(),
    install_requires=[
        "grpcio>=1.60.0",
        "grpcio-tools>=1.60.0",
        "protobuf>=4.25.1",
        "torch>=2.0.0",
        "transformers>=4.35.0",
        "Pillow>=10.0.0",
        "numpy>=1.24.0",
        "accelerate>=0.25.0",
    ],
    python_requires=">=3.8",
)
