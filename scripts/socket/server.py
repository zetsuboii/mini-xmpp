import socket

# Set up the socket
server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server_socket.bind(('0.0.0.0', 9292))
server_socket.listen(1)

print("Listening on port 9292...")

# Accept a connection
client_socket, client_address = server_socket.accept()
print(f"Connection from {client_address}")

# Receive and print the message
while True:
    message = client_socket.recv(1024).decode()
    if not message:
        break
    print(f"Received message: {message}")

# Close the connection
client_socket.close()
server_socket.close()