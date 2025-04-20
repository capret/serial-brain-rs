import math
import serial
import socket
import time
import numpy as np
import logging
from PySide6.QtCore import QThread, Signal
from collections import deque
from .base_reader import BaseReader
logger = logging.getLogger(__name__)

# -------------------------------
# SocketReader implements reading from a TCP socket.
# -------------------------------
class SocketReader(BaseReader):
    def __init__(self, host="0.0.0.0", port=8234, parent=None):
        super().__init__(parent)
        self.socket_host = host
        self.socket_port = port
        self.server_socket = None
        self.client_socket = None

    def setup_connection(self):
        self.server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.server_socket.bind((self.socket_host, self.socket_port))
        self.server_socket.listen(1)
        self.logger.info(f"SocketReader listening on {self.socket_host}:{self.socket_port}")
        self.client_socket, addr = self.server_socket.accept()
        self.logger.info(f"SocketReader connected by: {addr}")
        self.client_socket.settimeout(0.1)
        self.connected = True

    def read_new_data(self) -> bytes:
        try:
            data = self.client_socket.recv(1024)
            if not data:
                # Client disconnected.
                self.running = False
            return data
        except socket.timeout:
            return b""

    def close_connection(self):
        if self.client_socket:
            try:
                self.client_socket.close()
            except Exception:
                pass
            self.client_socket = None
        if self.server_socket:
            try:
                self.server_socket.close()
            except Exception:
                pass
            self.server_socket = None
        self.connected = False