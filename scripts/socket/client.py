import socket

# Set up the socket
client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

# Connect to the server
server_address = ('172.18.0.2', 9292)  # Replace '<server_ip>' with the server's IP address
client_socket.connect(server_address)

# Send a message
message = "Hello, Server!"
client_socket.sendall(message.encode())

# Close the socket
client_socket.close()
