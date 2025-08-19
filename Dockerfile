FROM rust:latest

# Install ffmpeg
RUN apt-get update && apt-get install -y ffmpeg

# Copy the project files
COPY . .

# Run the tests
CMD ["cargo", "test"]
