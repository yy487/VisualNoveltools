# -*- coding: utf-8 -*-
"""Silky Engine LZSS codec.

This implementation is adapted from TesterTesterov/SilkyArcTool and kept as a
small standalone module so archive code can stay focused on container logic.
"""


class SilkyLZSS:
    def __init__(self, buffer: bytes, N: int = 4096, F: int = 18, threshold: int = 2,
                 null: int | None = None, padding_byte: bytes | int = b"\x00"):
        self.input_buffer = buffer
        self.null = N if null is None else null
        self.padding_byte = padding_byte if isinstance(padding_byte, int) else padding_byte[0]
        self.N = N
        self.F = F
        self.threshold = threshold
        self.text_buffer = [0] * (self.N + self.F - 1)
        self.match_position = 0
        self.match_length = 0
        self.lson = [0] * (self.N + 1)
        self.rson = [0] * (self.N + 257)
        self.dad = [0] * (self.N + 1)

    def init_tree(self) -> None:
        for i in range(self.N + 1, self.N + 257):
            self.rson[i] = self.null
        for i in range(self.N):
            self.dad[i] = self.null

    def insert_node(self, r: int) -> None:
        cmp = 1
        p = self.N + 1 + self.text_buffer[r]
        self.rson[r] = self.null
        self.lson[r] = self.null
        self.match_length = 0
        while True:
            if cmp >= 0:
                if self.rson[p] != self.null:
                    p = self.rson[p]
                else:
                    self.rson[p] = r
                    self.dad[r] = p
                    return
            else:
                if self.lson[p] != self.null:
                    p = self.lson[p]
                else:
                    self.lson[p] = r
                    self.dad[r] = p
                    return
            i = 0
            for i in range(1, self.F):
                cmp = self.text_buffer[r + i] - self.text_buffer[p + i]
                if cmp != 0:
                    i -= 1
                    break
            i += 1
            if i > self.match_length:
                self.match_position = p
                self.match_length = i
                if self.match_length >= self.F:
                    break
        self.dad[r] = self.dad[p]
        self.lson[r] = self.lson[p]
        self.rson[r] = self.rson[p]
        self.dad[self.lson[p]] = r
        self.dad[self.rson[p]] = r
        if self.rson[self.dad[p]] == p:
            self.rson[self.dad[p]] = r
        else:
            self.lson[self.dad[p]] = r
        self.dad[p] = self.null

    def delete_node(self, p: int) -> None:
        if self.dad[p] == self.null:
            return
        if self.rson[p] == self.null:
            q = self.lson[p]
        elif self.lson[p] == self.null:
            q = self.rson[p]
        else:
            q = self.lson[p]
            if self.rson[q] != self.null:
                q = self.rson[q]
                while self.rson[q] != self.null:
                    q = self.rson[q]
                self.rson[self.dad[q]] = self.lson[q]
                self.dad[self.lson[q]] = self.dad[q]
                self.lson[q] = self.lson[p]
                self.dad[self.lson[p]] = q
            self.rson[q] = self.rson[p]
            self.dad[self.rson[p]] = q
        self.dad[q] = self.dad[p]
        if self.rson[self.dad[p]] == p:
            self.rson[self.dad[p]] = q
        else:
            self.lson[self.dad[p]] = q
        self.dad[p] = self.null

    def encode(self) -> bytes:
        r = self.N - self.F
        s = 0
        code_buf = [0] * 17
        mask = 1
        code_buf_ptr = 1
        output = bytearray()
        self.init_tree()
        code_buf[0] = 0
        for i in range(s, r):
            self.text_buffer[i] = self.padding_byte
        length = 0
        for length in range(0, self.F):
            if length >= len(self.input_buffer):
                length -= 1
                break
            self.text_buffer[r + length] = self.input_buffer[length]
        length += 1
        if length == 0:
            return b""
        for i in range(1, self.F + 1):
            self.insert_node(r - i)
        self.insert_node(r)
        pos = i
        while True:
            if self.match_length > length:
                self.match_length = length
            if self.match_length <= self.threshold:
                self.match_length = 1
                code_buf[0] |= mask
                code_buf[code_buf_ptr] = self.text_buffer[r]
                code_buf_ptr += 1
            else:
                code_buf[code_buf_ptr] = self.match_position & 0xff
                code_buf_ptr += 1
                code_buf[code_buf_ptr] = (((self.match_position >> 4) & 0xf0) |
                                          (self.match_length - (self.threshold + 1))) & 0xff
                code_buf_ptr += 1
            mask = (mask << 1) & 0xff
            if mask == 0:
                output.extend(code_buf[:code_buf_ptr])
                code_buf[0] = 0
                code_buf_ptr = 1
                mask = 1
            last_match_length = self.match_length
            i = 0
            for i in range(0, last_match_length):
                if pos >= len(self.input_buffer):
                    i -= 1
                    break
                self.delete_node(s)
                c = self.input_buffer[pos]
                pos += 1
                self.text_buffer[s] = c
                if s < (self.F - 1):
                    self.text_buffer[s + self.N] = c
                s = (s + 1) & (self.N - 1)
                r = (r + 1) & (self.N - 1)
                self.insert_node(r)
            i += 1
            while i < last_match_length:
                i += 1
                self.delete_node(s)
                s = (s + 1) & (self.N - 1)
                r = (r + 1) & (self.N - 1)
                length -= 1
                if length:
                    self.insert_node(r)
            i += 1
            if length <= 0:
                break
        if code_buf_ptr > 1:
            output.extend(code_buf[:code_buf_ptr])
        return bytes(output)

    def decode(self) -> bytes:
        output = bytearray()
        r = self.N - self.F
        flags = 0
        self.init_tree()
        for i in range(0, r):
            self.text_buffer[i] = self.padding_byte
        current_pos = 0
        while True:
            flags >>= 1
            if (flags & 256) == 0:
                if current_pos >= len(self.input_buffer):
                    break
                c = self.input_buffer[current_pos]
                current_pos += 1
                flags = c | 0xff00
            if flags & 1:
                if current_pos >= len(self.input_buffer):
                    break
                c = self.input_buffer[current_pos]
                current_pos += 1
                output.append(c)
                self.text_buffer[r] = c
                r = (r + 1) & (self.N - 1)
            else:
                if current_pos >= len(self.input_buffer):
                    break
                i = self.input_buffer[current_pos]
                current_pos += 1
                if current_pos >= len(self.input_buffer):
                    break
                j = self.input_buffer[current_pos]
                current_pos += 1
                i |= (j & 0xf0) << 4
                j = (j & 0x0f) + self.threshold
                for k in range(0, j + 1):
                    c = self.text_buffer[(i + k) & (self.N - 1)]
                    output.append(c)
                    self.text_buffer[r] = c
                    r = (r + 1) & (self.N - 1)
        return bytes(output)


def compress(data: bytes) -> bytes:
    return SilkyLZSS(data).encode()


def decompress(data: bytes) -> bytes:
    return SilkyLZSS(data).decode()
