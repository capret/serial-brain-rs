import serial
import logging
logger = logging.getLogger(__name__)
from .base_reader import BaseReader


# -------------------------------
# SerialReader implements reading from a serial port.
# -------------------------------
class SerialReader(BaseReader):
    def __init__(self, port, baudrate, stopbits, parity, bytesize, parent=None):
        super().__init__(parent)
        self.port = port
        self.baudrate = baudrate
        self.stopbits = stopbits
        self.parity = parity
        self.bytesize = bytesize
        self.serial_port: serial.Serial = None
        logger.info(f"SerialReader initialized on {self.port} with baudrate {self.baudrate}, stopbits {self.stopbits}, parity {self.parity}, bytesize {self.bytesize}")

    def setup_connection(self):
        self.serial_port = serial.Serial(
            port=self.port,
            baudrate=self.baudrate,
            stopbits=self.stopbits,
            parity=self.parity,
            bytesize=self.bytesize,
            timeout=0.1
        )
        self.connected = True
        self.logger.info("SerialReader connection established")

    def read_new_data(self) -> bytes:
        if self.serial_port.in_waiting > 0:
            return self.serial_port.read(self.serial_port.in_waiting)
        return b""

    def close_connection(self):
        if self.serial_port and self.serial_port.is_open:
            try:
                self.serial_port.close()
            except Exception:
                pass
            self.serial_port = None
        self.connected = False