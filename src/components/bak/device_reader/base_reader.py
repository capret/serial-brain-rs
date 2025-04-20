import math
import time
import numpy as np
import logging
from PySide6.QtCore import QThread, Signal
from collections import deque
logger = logging.getLogger(__name__)

class BaseReader(QThread):
    # Existing signals.
    data_packet_received = Signal(bytes)
    info_report_received = Signal(bytes)
    timed_packet_received = Signal(bytes, float)
    connection_status = Signal(bool)
    error_occurred = Signal(str)
    # New signal to report the checksum failure rate (in percent) over the recent window.
    checksum_failed_rate_calculated = Signal(float)

    connected = False
    # Constants for packet handling.
    DATA_HEADER = b'\xAA\xFF\xF1\x20'
    DATA_PACKET_LENGTH = 38  # 4(header) + 32(data) + 2(checksum)

    def __init__(self, rolling_window_size=100, parent=None):
        """
        rolling_window_size: Number of most recent packets to consider when calculating the failure rate.
        """
        super().__init__(parent)
        self.running = False
        self.fake_data_enabled = False
        self.fake_t = 0.0
        self.buffer = bytearray()
        self.logger = logging.getLogger(self.__class__.__name__)
        # Rolling window for packet status: 0 for success, 1 for checksum failure.
        self.rolling_window_size = rolling_window_size
        self.recent_packet_status = deque(maxlen=200)

    def compute_checksum(self, data):
        arr = np.frombuffer(data[:36], dtype=np.uint8)
        checksum1 = int(arr.sum() % 256)
        prefix = np.cumsum(arr, dtype=np.uint16) % 256
        checksum2 = int(prefix.sum() % 256)
        return checksum1, checksum2

    def verify_packet(self, packet: bytes) -> bool:
        if len(packet) != self.DATA_PACKET_LENGTH:
            return False
        sc1, sc2 = self.compute_checksum(bytearray(packet))
        return packet[36] == sc1 and packet[37] == sc2

    def generate_fake_packet(self) -> bytes:
        packet = bytearray(self.DATA_PACKET_LENGTH)
        packet[0:4] = self.DATA_HEADER
        for i in range(8):
            phase_shift = (math.pi / 4) * i
            sine_value = math.sin(self.fake_t + phase_shift)
            value = int((sine_value + 1) * 50)
            start = 4 + i * 4
            packet[start:start+4] = value.to_bytes(4, byteorder='little', signed=True)
        packet[36], packet[37] = self.compute_checksum(packet)
        self.fake_t += 0.2
        return bytes(packet)

    def get_check_failed_rate(self):
        if len(self.recent_packet_status) > 0:
            failure_rate = (sum(self.recent_packet_status) / len(self.recent_packet_status)) * 100.0
            return failure_rate
        return 0

    def process_buffer(self):
        """Process self.buffer and emit signals for complete packets or info reports."""
        i = 0
        buf_len = len(self.buffer)
        while i < buf_len:
            # Look for a packet header.
            if buf_len - i >= 4 and self.buffer[i:i+4] == self.DATA_HEADER:
                if buf_len - i >= self.DATA_PACKET_LENGTH:
                    packet = bytes(self.buffer[i:i+self.DATA_PACKET_LENGTH])
                    timestamp = time.time()
                    # Check if the packet's checksum is valid.
                    if self.verify_packet(packet):
                        self.recent_packet_status.append(0)
                        self.timed_packet_received.emit(packet, timestamp)
                    else:
                        self.recent_packet_status.append(1)
                        self.logger.error("Checksum error in received packet")
                    i += self.DATA_PACKET_LENGTH
                    continue
                else:
                    break  # Not enough data yet.
            else:
                # Process an info report delimited by newline.
                newline_index = self.buffer.find(b'\n', i)
                header_index = self.buffer.find(self.DATA_HEADER, i)
                if newline_index != -1 and (header_index == -1 or newline_index < header_index):
                    report = bytes(self.buffer[i:newline_index+1])
                    self.info_report_received.emit(report)
                    i = newline_index + 1
                elif header_index != -1:
                    if header_index > i:
                        report = bytes(self.buffer[i:header_index])
                        self.info_report_received.emit(report)
                    i = header_index
                else:
                    break
            buf_len = len(self.buffer)
        if i > 0:
            self.buffer = self.buffer[i:]

    # The following methods must be implemented by subclasses.
    def setup_connection(self):
        raise NotImplementedError("Subclasses must implement setup_connection()")

    def read_new_data(self) -> bytes:
        raise NotImplementedError("Subclasses must implement read_new_data()")

    def close_connection(self):
        raise NotImplementedError("Subclasses must implement close_connection()")

    def run(self):
        self.logger.info("Reader started")
        try:
            self.setup_connection()
            self.connection_status.emit(True)
            self.running = True
            while self.running:
                if self.fake_data_enabled:
                    packet = self.generate_fake_packet()
                    timestamp = time.time()
                    self.buffer.extend(packet)
                    self.process_buffer()
                    self.msleep(10)
                else:
                    try:
                        data = self.read_new_data()
                        if data:
                            self.buffer.extend(data)
                            self.process_buffer()
                        else:
                            self.msleep(1)
                    except Exception as e:
                        self.error_occurred.emit(f"Exception: {e}")
                        self.connection_status.emit(False)
                        self.running = False
        except Exception as e:
            self.logger.warning(f"Connection setup error: {e}")
            self.error_occurred.emit(f"Connection setup error: {e}")
            self.connection_status.emit(False)
        finally:
            self.close_connection()
            self.connection_status.emit(False)
            self.logger.info("Reader finished")

    def stop(self):
        self.logger.info("Stopping reader")
        self.running = False
        self.close_connection()
        self.wait()