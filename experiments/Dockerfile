# Use an official Ubuntu base image
FROM ubuntu:jammy

# Install SSH server
RUN apt-get update && apt-get install -y openssh-server
RUN mkdir /var/run/sshd

# Set root password (change 'your_password' to a secure password)
RUN echo 'root:password' | chpasswd

# SSH login fix. Otherwise, the user is kicked off after login
RUN sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config

# Expose the SSH port
EXPOSE 22

# Run the SSH server
CMD ["/usr/sbin/sshd", "-D"]
