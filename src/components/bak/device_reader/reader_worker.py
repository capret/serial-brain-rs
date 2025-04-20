import logging
logger = logging.getLogger(__name__)
from .serial_reader import SerialReader
from .socket_reader import SocketReader

# -------------------------------
# ReaderWorker is a simple wrapper that chooses the reader based on a mode.
# -------------------------------
class ReaderWorker:
    def __init__(self, mode: str = "serial", **kwargs):
        """
        mode: "serial" or "socket"
        kwargs: arguments passed to the respective reader.
          For serial mode, expect: port, baudrate, stopbits, parity, bytesize.
          For socket mode, you can pass: host and port.
        """
        if mode == "socket":
            self.reader = SocketReader(**kwargs)
            logger.info(f"SocketReader listening on 0.0.0.0:{kwargs['port']}")
        else:
            self.reader = SerialReader(**kwargs)
            logger.info(f"SerialReader connected to {kwargs['port']}")

    def start(self):
        logger.info("Reader started - ReaderWorker")
        self.reader.start()
        

    def stop(self):
        logger.info("Reader stopped - ReaderWorker")
        self.reader.stop()
        

    # Expose signals from the underlying reader.
    @property
    def data_packet_received(self):
        return self.reader.data_packet_received

    @property
    def info_report_received(self):
        return self.reader.info_report_received

    @property
    def timed_packet_received(self):
        return self.reader.timed_packet_received

    @property
    def connection_status(self):
        return self.reader.connection_status

    @property
    def error_occurred(self):
        return self.reader.error_occurred
    
    def enable_fake_data(self, enable):
        self.reader.fake_data_enabled = enable
        if hasattr(self.reader, 'serial_port') and self.reader.serial_port:
                self.reader.serial_port.reset_input_buffer()

        self.reader.buffer.clear()

