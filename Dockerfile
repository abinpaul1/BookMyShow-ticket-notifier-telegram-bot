FROM rust:1.57

COPY ./ ./

# Build your program
RUN mkdir db
RUN cargo build --release

# Run the binary
CMD ["./target/release/book_my_show_notifier_bot"]