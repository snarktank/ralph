import socket
import random
from typing import Set


class PortManager:
    """Manages port allocation for projects"""

    def __init__(self, port_range_start: int = 3000, port_range_end: int = 9000):
        self.port_range_start = port_range_start
        self.port_range_end = port_range_end
        self.allocated_ports: Set[int] = set()

    def is_port_available(self, port: int) -> bool:
        """Check if a port is available"""
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.bind(('', port))
                return True
        except OSError:
            return False

    def allocate_port(self) -> int:
        """Allocate a random available port"""
        max_attempts = 100
        for _ in range(max_attempts):
            port = random.randint(self.port_range_start, self.port_range_end)
            if port not in self.allocated_ports and self.is_port_available(port):
                self.allocated_ports.add(port)
                return port

        raise RuntimeError("Could not find available port")

    def release_port(self, port: int):
        """Release a port"""
        self.allocated_ports.discard(port)


# Global instance
port_manager = PortManager()
