#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""C148: peers mTLS e MQTT 5 wire independentes, sem brokers ou estado reais."""

import argparse
from contextlib import contextmanager
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import secrets
import signal
import socket
import ssl
import subprocess
import sys
import tempfile
import time
import traceback

sys.dont_write_bytecode = True
PATH_PROPERTY = "xmqr.bridge.path"
BRIDGE_ID = "wire-c148"
PROPERTY_TYPES = {
    0x01: "byte", 0x02: "u32", 0x03: "text", 0x08: "text", 0x09: "binary",
    0x0B: "vbi", 0x11: "u32", 0x12: "text", 0x13: "u16", 0x15: "text",
    0x16: "binary", 0x17: "byte", 0x18: "u32", 0x19: "byte", 0x1A: "text",
    0x1C: "text", 0x1F: "text", 0x21: "u16", 0x22: "u16", 0x23: "u16",
    0x24: "byte", 0x25: "byte", 0x26: "pair", 0x27: "u32", 0x28: "byte",
    0x29: "byte", 0x2A: "byte",
}


class CheckError(Exception):
    """Erro com diagnóstico controlado: nunca inclui CONNECT ou credenciais."""


def require(condition, message):
    if not condition:
        raise CheckError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def private(path, contents):
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o600)


def field(value):
    value = value.encode("utf-8") if isinstance(value, str) else value
    return len(value).to_bytes(2, "big") + value


def vbi(value):
    require(0 <= value <= 268435455, "VBI da fixture fora do limite")
    encoded = bytearray()
    while True:
        byte = value % 128
        value //= 128
        encoded.append(byte | (128 if value else 0))
        if not value:
            return bytes(encoded)


def packet(header, body):
    return bytes([header]) + vbi(len(body)) + body


class Cursor:
    def __init__(self, data):
        self.data = data
        self.offset = 0

    def take(self, count):
        require(count >= 0 and self.offset + count <= len(self.data), "pacote wire truncado")
        result = self.data[self.offset:self.offset + count]
        self.offset += count
        return result

    def number(self, count=1):
        return int.from_bytes(self.take(count), "big")

    def variable(self):
        value = 0
        for index in range(4):
            byte = self.number()
            value += (byte & 127) << (7 * index)
            if byte < 128:
                require(index == 0 or byte != 0, "VBI wire não mínimo")
                return value
        raise CheckError("VBI wire acima de quatro bytes")

    def binary(self):
        return self.take(self.number(2))

    def text(self):
        return self.binary().decode("utf-8")

    def properties(self):
        length = self.variable()
        end = self.offset + length
        require(end <= len(self.data), "propriedades wire truncadas")
        result = []
        while self.offset < end:
            identifier = self.variable()
            kind = PROPERTY_TYPES.get(identifier)
            require(kind is not None, "propriedade MQTT desconhecida na fixture")
            if kind == "byte":
                value = self.number()
            elif kind in ("u16", "u32"):
                value = self.number(2 if kind == "u16" else 4)
            elif kind == "text":
                value = self.text()
            elif kind == "binary":
                value = self.binary()
            elif kind == "pair":
                value = (self.text(), self.text())
            else:
                value = self.variable()
            require(self.offset <= end, "propriedade ultrapassou o bloco MQTT")
            result.append((identifier, value))
        require(self.offset == end, "comprimento do bloco MQTT inválido")
        return result


def properties(items):
    result = bytearray()
    for identifier, value in items:
        result.extend(vbi(identifier))
        kind = PROPERTY_TYPES[identifier]
        if kind in ("byte", "u16", "u32"):
            result.extend(value.to_bytes({"byte": 1, "u16": 2, "u32": 4}[kind], "big"))
        elif kind in ("text", "binary"):
            result.extend(field(value))
        elif kind == "pair":
            result.extend(field(value[0]) + field(value[1]))
        else:
            result.extend(vbi(value))
    return vbi(len(result)) + result


def publish(topic="src/value", payload=b"sample", qos=1, pkid=37,
            retain=False, props=(), duplicate=False):
    body = field(topic)
    if qos:
        body += pkid.to_bytes(2, "big")
    body += properties(props) + payload
    return packet(0x30 | (8 if duplicate else 0) | qos * 2 | int(retain), body)


def acknowledgement(kind, pkid, reason=0):
    return packet(kind | (2 if kind == 0x60 else 0), pkid.to_bytes(2, "big") + bytes([reason, 0]))


def exact(stream, count):
    result = bytearray()
    while len(result) < count:
        chunk = stream.recv(count - len(result))
        if not chunk:
            raise EOFError("peer TLS fechado")
        result.extend(chunk)
    return bytes(result)


def receive(stream, timeout=5):
    previous = stream.gettimeout()
    stream.settimeout(timeout)
    try:
        while True:
            header = exact(stream, 1)[0]
            remaining = 0
            for index in range(4):
                byte = exact(stream, 1)[0]
                remaining += (byte & 127) << (index * 7)
                if byte < 128:
                    break
            else:
                raise CheckError("Remaining Length wire acima do limite")
            require(remaining <= 131072, "peer enviou pacote acima do limite da fixture")
            body = exact(stream, remaining)
            if header == 0xC0:
                require(not body, "PINGREQ inválido")
                stream.sendall(b"\xd0\x00")
                continue
            return header, body
    finally:
        stream.settimeout(previous)


def quiet(stream, seconds=0.18, closed=False):
    try:
        received = receive(stream, seconds)
    except (TimeoutError, socket.timeout):
        return
    except (EOFError, ConnectionError, ssl.SSLError):
        require(closed, "perna TLS encerrou antes da confirmação esperada")
        return
    raise CheckError(f"pacote inesperado durante ausência de ACK/entrega: tipo0x{received[0]:02x}")


def read_publish(stream):
    header, body = receive(stream)
    require(header >> 4 == 3, f"esperado PUBLISH; recebido tipo0x{header:02x}")
    cursor = Cursor(body)
    qos = (header >> 1) & 3
    require(qos <= 2, "QoS reservado recebido")
    topic = cursor.text()
    pkid = cursor.number(2) if qos else 0
    props = cursor.properties()
    return dict(topic=topic, qos=qos, pkid=pkid, retain=bool(header & 1),
                duplicate=bool(header & 8), properties=props,
                payload=cursor.take(len(body) - cursor.offset), size=1 + len(vbi(len(body))) + len(body))


def read_ack(stream, kind, pkid, reason=0):
    header, body = receive(stream)
    require(header == (kind | (2 if kind == 0x60 else 0)), "ACK MQTT de tipo inesperado")
    cursor = Cursor(body)
    require(cursor.number(2) == pkid, "ACK MQTT com identificador divergente")
    actual = cursor.number() if cursor.offset < len(body) else 0
    require(actual == reason, f"reason code de ACK divergente:0x{actual:02x}")
    if cursor.offset < len(body):
        cursor.properties()
    require(cursor.offset == len(body), "bytes após ACK MQTT")


def values(packet_data, identifier):
    return [value for key, value in packet_data["properties"] if key == identifier]


class Peer:
    def __init__(self, fixture):
        self.fixture = fixture
        self.listener = socket.socket()
        self.listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.listener.bind(("127.0.0.1", 0))
        self.listener.listen(4)
        self.listener.settimeout(5)
        self.port = self.listener.getsockname()[1]
        self.context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        self.context.minimum_version = ssl.TLSVersion.TLSv1_2
        self.context.load_cert_chain(str(fixture.root / "server.crt"), str(fixture.root / "server.key"))
        self.context.load_verify_locations(cafile=str(fixture.root / "ca.crt"))
        self.context.verify_mode = ssl.CERT_REQUIRED
        self.connections = []

    def connect(self, expected_id, session=False, caps=(), reason=0):
        raw, address = self.listener.accept()
        require(address[0] == "127.0.0.1", "conexão fora de loopback")
        raw.settimeout(5)
        stream = self.context.wrap_socket(raw, server_side=True)
        self.connections.append(stream)
        require(hashlib.sha256(stream.getpeercert(binary_form=True)).hexdigest() == self.fixture.fingerprint,
                "certificado cliente inesperado")
        header, body = receive(stream)
        require(header == 0x10, "primeiro pacote não é CONNECT")
        cursor = Cursor(body)
        require(cursor.text() == "MQTT" and cursor.number() == 5, "peer não negociou MQTT5")
        flags = cursor.number()
        require(flags & 0xC0 == 0xC0 and flags & 0x3F == 0, "CONNECT não persistente ou flags inesperadas")
        require(cursor.number(2) == 30, "keepalive do CONNECT divergente")
        props = cursor.properties()
        require(cursor.text() == expected_id, "identidade de client não corresponde à rota")
        require(cursor.text() == "wire-device", "username de teste divergente")
        require(cursor.binary() == self.fixture.secret.encode(), "password de teste divergente")
        require(cursor.offset == len(body), "bytes inesperados após CONNECT")
        require((0x11, 86400) in props and (0x21, 1) in props, "sessão durável/Receive Maximum não solicitado")
        stream.sendall(packet(0x20, bytes([int(session), reason]) + properties(caps)))
        return stream, props

    def close(self):
        for connection in self.connections:
            try:
                connection.close()
            except OSError:
                pass
        self.listener.close()


class Case:
    def __init__(self, fixture, name, route_qos=1, policy="preserve", max_hops=4):
        self.fixture = fixture
        self.root = fixture.root / name
        self.root.mkdir(mode=0o700)
        self.state = self.root / "state"
        self.state.mkdir(mode=0o700)
        self.journal = self.state / "route/bridge-forward.json"
        self.source_peer = Peer(fixture)
        self.destination_peer = Peer(fixture)
        self.qos = route_qos
        self.source = self.destination = self.child = None
        self.expect_failed = False
        self.checked = False
        self.log = self.root / "bridge.log"
        topology = "version=1\n"
        for broker, peer in [("origin", self.source_peer), ("target", self.destination_peer)]:
            topology += f'[[brokers]]\nid="{broker}"\nhost="127.0.0.1"\nport={peer.port}\n'
        topology += ('[[links]]\nid="route"\nsource_broker="origin"\ndestination_broker="target"\n'
                     f'source_prefix="src/"\ntarget_prefix="dst/"\nqos={route_qos}\n')
        private(self.root / "topology.toml", topology)
        config = (f'version=2\nsource_protocol="mqtt5"\ndestination_protocol="mqtt5"\n'
                  f'bridge_id="{BRIDGE_ID}"\nmax_hops={max_hops}\n'
                  f'response_topic_policy="{policy}"\nmax_retries=1\n'
                  f'topology_file={json.dumps(str(self.root / "topology.toml"))}\n'
                  f'state_dir={json.dumps(str(self.state))}\n')
        for endpoint, peer, identity in [("source", self.source_peer, "origin"),
                                         ("destination", self.destination_peer, "target")]:
            config += (f'[{endpoint}]\nbroker_id="{identity}"\nhost="127.0.0.1"\n'
                       f'port={peer.port}\nclient_id="c148-{identity}"\nusername="wire-device"\n')
            for key, filename in [("password_file", "password"), ("ca", "ca.crt"),
                                  ("cert", "client.crt"), ("key", "client.key")]:
                config += f'{key}={json.dumps(str(fixture.root / filename))}\n'
        config += f'[[routes]]\nid="route"\nsource_prefix="src/"\ntarget_prefix="dst/"\nqos={route_qos}\n'
        self.config = self.root / "bridge.toml"
        private(self.config, config)

    def launch(self, checkpoint=None):
        if not self.checked:
            result = subprocess.run([str(self.fixture.bridge), "--check", "--config", str(self.config)],
                                    capture_output=True, timeout=8)
            require(self.fixture.secret.encode() not in result.stdout + result.stderr,
                    "credencial de fixture apareceu no diagnóstico")
            require(result.returncode == 0, f"preflight --check recusou template de fixture: exit{result.returncode}")
            require(not list(self.state.iterdir()), "--check modificou diretório de estado")
            self.checked = True
        env = dict(os.environ)
        env.pop("XMQR_BRIDGE_TEST_CHECKPOINT", None)
        if checkpoint is not None:
            env["XMQR_BRIDGE_TEST_CHECKPOINT"] = checkpoint
        with self.log.open("ab") as log:
            self.child = subprocess.Popen([str(self.fixture.bridge), "--config", str(self.config)],
                                          stdout=log, stderr=log, env=env)

    def destination_connect(self, session=False, caps=()):
        self.destination, self.destination_connect_props = self.destination_peer.connect(
            "c148-target-route", session=session, caps=caps)

    def source_connect(self, session=False):
        self.source, self.source_connect_props = self.source_peer.connect("c148-origin-route", session=session)
        header, body = receive(self.source)
        require(header == 0x82, "esperado SUBSCRIBE da origem")
        cursor = Cursor(body)
        identifier = cursor.number(2)
        self.subscribe_props = cursor.properties()
        require(cursor.text() == "src/#", "filtro da assinatura divergente")
        options = cursor.number()
        require(options & 3 == self.qos and options & 8 and options & 0x30 == 0x10,
                "QoS/Retain As Published/retain handling da assinatura divergente")
        require(cursor.offset == len(body), "mais de um filtro inesperado")
        self.source.sendall(packet(0x90, identifier.to_bytes(2, "big") + b"\x00" + bytes([self.qos])))

    def start(self, destination_session=False, source_session=False, caps=(), checkpoint=None):
        self.launch(checkpoint)
        self.destination_connect(destination_session, caps)
        self.source_connect(source_session)
        self.wait_journal(lambda document: document["source_known"] and document["destination_known"])

    def document(self):
        require(self.journal.stat().st_mode & 0o077 == 0, "journal sem privacidade0600")
        require(self.journal.parent.stat().st_mode & 0o077 == 0, "diretório de rota sem privacidade0700")
        container = json.loads(self.journal.read_text())
        require(container["version"] == 2, "versão de journal divergente")
        return container["document"]

    def wait_journal(self, predicate, timeout=3):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            try:
                document = self.document()
                if predicate(document):
                    return document
            except FileNotFoundError:
                pass
            time.sleep(0.02)
        raise CheckError("journal não atingiu a fase durável esperada")

    def phase(self, phase):
        return self.wait_journal(lambda d: d["pending"] is not None and d["pending"]["phase"] == phase)

    def finish(self, source_qos, source_pkid=37):
        if self.qos:
            self.phase("sent")
            quiet(self.source)
        if self.qos == 1:
            self.destination.sendall(acknowledgement(0x40, 1))
        elif self.qos == 2:
            self.destination.sendall(acknowledgement(0x50, 1))
            read_ack(self.destination, 0x60, 1)
            self.phase("rel")
            quiet(self.source)
            self.destination.sendall(acknowledgement(0x70, 1))
        if source_qos == 1:
            read_ack(self.source, 0x40, source_pkid)
        elif source_qos == 2:
            read_ack(self.source, 0x50, source_pkid)
            self.phase("done")
            self.source.sendall(acknowledgement(0x60, source_pkid))
            read_ack(self.source, 0x70, source_pkid)
        self.wait_journal(lambda d: d["pending"] is None)

    def discarded(self, source_qos=1, pkid=37):
        quiet(self.destination)
        if source_qos == 1:
            read_ack(self.source, 0x40, pkid)
        elif source_qos == 2:
            read_ack(self.source, 0x50, pkid)
            self.phase("done")
            self.source.sendall(acknowledgement(0x60, pkid))
            read_ack(self.source, 0x70, pkid)
        self.wait_journal(lambda d: d["pending"] is None)
        quiet(self.destination)

    def failed(self, phase=None):
        self.expect_failed = True
        require(self.child.wait(timeout=8) != 0, "erro de protocolo/capacidade encerrou com sucesso")
        quiet(self.source, closed=True)
        quiet(self.destination, closed=True)
        if phase:
            self.phase(phase)

    def stop(self, crash=False):
        if self.child and self.child.poll() is None:
            self.child.send_signal(signal.SIGKILL if crash else signal.SIGINT)
            try:
                result = self.child.wait(timeout=5)
                if not crash:
                    require(result == 0, "parada própria da bridge não terminou com sucesso")
            except subprocess.TimeoutExpired:
                self.child.kill()
                self.child.wait(timeout=5)
                if not crash:
                    raise CheckError("bridge não encerrou após SIGINT")

    def close(self):
        try:
            self.stop(crash=True)
        finally:
            self.source_peer.close()
            self.destination_peer.close()
            if self.log.exists():
                require(self.fixture.secret.encode() not in self.log.read_bytes(), "credencial de fixture apareceu em log")


class Fixture:
    def __init__(self, bridge, root):
        self.bridge, self.root = bridge, root
        self.secret = secrets.token_urlsafe(24)
        private(root / "password", self.secret + "\n")
        certificate = ssl.PEM_cert_to_DER_cert((root / "client.crt").read_text())
        self.fingerprint = hashlib.sha256(certificate).hexdigest()

    @contextmanager
    def case(self, name, **options):
        case = Case(self, name, **options)
        try:
            yield case
            if not case.expect_failed:
                require(case.child.poll() is None, "bridge encerrou inesperadamente")
                case.stop()
        finally:
            case.close()


def qos_matrix(fixture, route_qos, source_qos):
    with fixture.case(f"qos-{route_qos}-{source_qos}", route_qos=route_qos) as case:
        case.start()
        for index, (retain, payload) in enumerate([(False, b"live"), (True, b"initial-retained"),
                                                   (True, b"live-retained"), (True, b"")]):
            pkid = 40 + index
            case.source.sendall(publish(payload=payload, qos=source_qos, pkid=pkid, retain=retain))
            received = read_publish(case.destination)
            require(received["topic"] == "dst/value" and received["qos"] == route_qos,
                    "tópico ou QoS de destino divergente")
            require(received["payload"] == payload and received["retain"] == retain,
                    "payload/retained/tombstone alterado")
            require(values(received, 0x26) == [(PATH_PROPERTY, BRIDGE_ID)], "percurso não inserido")
            case.finish(source_qos, pkid)
    return dict(route_qos=route_qos, source_qos=source_qos, messages=4,
                retained_initial=True, retained_live=True, tombstone=True)


def metadata_case(fixture, policy):
    with fixture.case("metadata-" + policy, route_qos=2, policy=policy) as case:
        case.start()
        response = "src/reply" if policy == "map-prefix" else "outside/reply"
        users = [("trace", "first"), (PATH_PROPERTY, "external"), ("trace", "second"), ("sensor", "dht22")]
        props = [(1, 1), (2, 30), (3, "application/json"), (8, response), (9, b"\x00\xff\x80X")]
        props += [(0x26, item) for item in users]
        payload = '{"temperatura":24,"unidade":"°C"}'.encode()
        case.source.sendall(publish(payload=payload, qos=2, retain=True, props=props))
        received = read_publish(case.destination)
        require(received["payload"] == payload and received["retain"], "payload ou retained de metadados alterado")
        for identifier, expected in [(1, [1]), (3, ["application/json"]),
                                     (8, ["dst/reply" if policy == "map-prefix" else response]),
                                     (9, [b"\x00\xff\x80X"])]:
            require(values(received, identifier) == expected, "metadado de aplicação não preservado")
        require(values(received, 0x26) == [("trace", "first"), ("trace", "second"),
                                          ("sensor", "dht22"), (PATH_PROPERTY, "external," + BRIDGE_ID)],
                "ordem/repetição de User Properties ou percurso alterado")
        require(len(values(received, 2)) == 1 and 1 <= values(received, 2)[0] <= 30, "expiry renovado ou ausente")
        require(not values(received, 0x23) and not values(received, 0x0B), "propriedade de conexão copiada")
        case.finish(2)
    return dict(response_policy=policy, ordered_duplicates=True, binary_correlation=True,
                expiry_not_increased=True, path_extended=True)


def discard_case(fixture, name, props, max_hops=4, qos=1):
    with fixture.case(name, route_qos=qos, max_hops=max_hops) as case:
        case.start()
        case.source.sendall(publish(qos=qos, props=props))
        case.discarded(qos)
    return dict(no_destination_publish=True, source_handshake_completed=True)


def rejected_message(fixture, name, props=(), payload=b"sample", policy="preserve"):
    with fixture.case(name, policy=policy) as case:
        case.start()
        case.source.sendall(publish(props=props, payload=payload))
        case.failed()
        require(case.document()["pending"] is None, "mensagem inválida foi admitida no journal")
    return dict(no_source_ack=True, no_destination_publish=True)


def capacity_case(fixture, name, caps, retain=False, payload=b"sample"):
    with fixture.case(name) as case:
        case.start(caps=caps)
        case.source.sendall(publish(retain=retain, payload=payload))
        case.failed("prepared")
    return dict(no_source_ack=True, no_destination_publish=True, preserved_phase="prepared")


def negative_ack(fixture, kind):
    with fixture.case(f"negative-ack-{kind:02x}", route_qos=1 if kind == 0x40 else 2) as case:
        case.start()
        case.source.sendall(publish(qos=2 if case.qos == 2 else 1))
        read_publish(case.destination)
        case.phase("sent")
        quiet(case.source)
        phase = "sent"
        if kind == 0x70:
            case.destination.sendall(acknowledgement(0x50, 1))
            read_ack(case.destination, 0x60, 1)
            case.phase("rel")
            quiet(case.source)
            phase = "rel"
        case.destination.sendall(acknowledgement(kind, 1, 0x92 if kind == 0x70 else 0x87))
        case.failed("refused")
    return dict(no_source_ack=True, no_destination_retry=True,
                phase_before_negative_ack=phase, preserved_phase="refused")


def pending_message(case, expiry=None):
    case.start()
    props = [] if expiry is None else [(2, expiry)]
    case.source.sendall(publish(props=props))
    received = read_publish(case.destination)
    case.phase("sent")
    quiet(case.source)
    return received


def expiry_recovery(fixture):
    with fixture.case("expiry-recovery") as case:
        first = pending_message(case, expiry=12)
        deadline = case.document()["pending"]["envelope"]["metadata"]["expires_ms"]
        initial = values(first, 2)[0]
        case.stop(crash=True)
        time.sleep(1.3)
        case.start(destination_session=True, source_session=True)
        replay = read_publish(case.destination)
        remaining = values(replay, 2)[0]
        require(replay["duplicate"] and replay["pkid"] == first["pkid"] == 1, "replay sem DUP/PID estável")
        require(0 <= remaining < initial <= 12, "expiry de replay renovado")
        require(case.document()["pending"]["envelope"]["metadata"]["expires_ms"] == deadline,
                "prazo absoluto do journal foi renovado")
        case.finish(1)
    return dict(initial_expiry=initial, replay_expiry=remaining, stable_pid=True,
                duplicate=True, absolute_deadline_preserved=True)


def missing_session(fixture, leg):
    with fixture.case("missing-session-" + leg) as case:
        pending_message(case)
        case.stop(crash=True)
        before = case.journal.read_bytes()
        case.launch()
        case.destination_connect(session=leg != "destination")
        if leg == "source":
            case.source, _ = case.source_peer.connect("c148-origin-route", session=False)
        case.failed("sent")
        require(case.journal.read_bytes() == before, "sessão ausente modificou journal pendente")
    return dict(missing_leg=leg, journal_byte_identical=True, no_source_ack=True)


def unknown_session(fixture, leg):
    with fixture.case("unknown-session-" + leg) as case:
        case.launch()
        case.destination_connect(session=leg == "destination")
        if leg == "source":
            case.source, _ = case.source_peer.connect("c148-origin-route", session=True)
        case.expect_failed = True
        require(case.child.wait(timeout=8) != 0, "sessão desconhecida foi aceita")
        document = case.document()
        expected_destination_known = leg == "source"
        require(not document["source_known"] and
                document["destination_known"] == expected_destination_known and
                document["pending"] is None,
                "sessão recusada adotou identidade indevida ou esqueceu a perna já aceita")
        quiet(case.destination, closed=True)
        if case.source:
            quiet(case.source, closed=True)
    return dict(unknown_leg=leg, source_known=document["source_known"],
                destination_known=document["destination_known"],
                rejected_leg_not_adopted=True, no_pending_delivery=True)


def qos2_duplicates(fixture):
    """MQTT-4.3.3-10: repetir PUBLISH antes de PUBREL não cria entrega nova."""
    with fixture.case("qos2-pending-duplicate", route_qos=2) as case:
        case.start()
        case.source.sendall(publish(qos=2))
        first = read_publish(case.destination)
        case.phase("sent")
        quiet(case.source)
        case.stop(crash=True)
        case.start(destination_session=True, source_session=True)
        replay = read_publish(case.destination)
        require(replay["duplicate"] and replay["pkid"] == first["pkid"] == 1,
                "replay do voo pendente não manteve DUP/PID")
        case.source.sendall(publish(qos=2, duplicate=True))
        quiet(case.source)
        quiet(case.destination)
        case.finish(2)
        require(any(item["pkid"] == 37 for item in case.document()["completed"]),
                "conclusão QoS2 não persistida")
        case.source.sendall(acknowledgement(0x60, 901))
        read_ack(case.source, 0x70, 901, 0x92)
        quiet(case.destination)
    return dict(pending_duplicate_no_extra_publish=True, recovery_replay_same_pid=True,
                unknown_pubrel_reason="0x92", mqtt_clause="MQTT-4.3.3-10")


def qos2_reuse(fixture, duplicate, different_payload):
    """MQTT-4.3.3-12: qualquer PUBLISH depois de PUBCOMP é mensagem nova."""
    suffix = f"dup{int(duplicate)}-{'different' if different_payload else 'same'}"
    with fixture.case("qos2-reuse-" + suffix, route_qos=2) as case:
        case.start()
        case.source.sendall(publish(qos=2))
        first = read_publish(case.destination)
        case.finish(2)
        if duplicate:
            # Uma nova publicação pode ter sido enviada pelo broker e perdida
            # antes de o client a observar. Após reconexão, sua primeira
            # observação local já pode ter DUP=1 e um PID agora reutilizado.
            case.stop(crash=True)
            case.start(destination_session=True, source_session=True)
        payload = b"new-application-message" if different_payload else b"sample"
        case.source.sendall(publish(qos=2, payload=payload, duplicate=duplicate))
        received = read_publish(case.destination)
        require(received["payload"] == payload and received["topic"] == first["topic"],
                "reuso de PID após PUBCOMP perdeu a nova mensagem")
        require(not received["duplicate"], "nova publicação reutilizou a fase remota concluída")
        case.finish(2)
    return dict(reused_source_pkid=37, source_duplicate=duplicate,
                different_payload=different_payload, new_message_forwarded=True,
                mqtt_clause="MQTT-4.3.3-12")


def optional_connection_properties(fixture):
    with fixture.case("connection-properties") as case:
        case.start()
        aliases = [value for key, value in case.source_connect_props if key == 0x22]
        identifiers = [value for key, value in case.subscribe_props if key == 0x0B]
        if not aliases or aliases[0] == 0:
            return dict(status="NOT_RUN", reason="CONNECT não anuncia Topic Alias Maximum; alias recebido não seria legal. Subscription Identifier também não foi solicitado.",
                        alias_negotiated=False, subscription_identifier_requested=bool(identifiers))
        case.source.sendall(publish(props=[(0x23, 1)]))
        received = read_publish(case.destination)
        require(not values(received, 0x23), "Topic Alias de origem foi copiado")
        case.finish(1)
    return dict(topic_alias_stripped=True, subscription_identifier_requested=bool(identifiers))


def scenarios(fixture):
    rows = []
    for route_qos in range(3):
        for source_qos in range(route_qos + 1):
            rows.append((f"qos_rota{route_qos}_origem{source_qos}_retained_tombstone",
                         lambda r=route_qos, s=source_qos: qos_matrix(fixture, r, s)))
    for policy in ["preserve", "map-prefix"]:
        rows.append(("metadados_" + policy, lambda p=policy: metadata_case(fixture, p)))
    rows.extend([
        ("expiry_zero_descarte", lambda: discard_case(fixture, "expiry-zero", [(2, 0)])),
        ("retorno_percurso_descarte_qos2", lambda: discard_case(fixture, "path-return", [(0x26, (PATH_PROPERTY, "external," + BRIDGE_ID))], qos=2)),
        ("teto_percurso_descarte", lambda: discard_case(fixture, "path-limit", [(0x26, (PATH_PROPERTY, "first,second"))], max_hops=2)),
        ("marcador_repetido_recusado", lambda: rejected_message(fixture, "path-repeated", [(0x26, (PATH_PROPERTY, "a")), (0x26, (PATH_PROPERTY, "b"))])),
        ("identidade_repetida_percurso_recusada", lambda: rejected_message(fixture, "path-duplicate-id", [(0x26, (PATH_PROPERTY, "a,a"))])),
        ("marcador_invalido_recusado", lambda: rejected_message(fixture, "path-invalid", [(0x26, (PATH_PROPERTY, "invalid id"))])),
        ("pfi_utf8_invalido_recusado", lambda: rejected_message(fixture, "pfi-invalid", [(1, 1)], payload=b"\xff")),
        ("response_fora_prefixo_recusado", lambda: rejected_message(fixture, "response-outside", [(8, "outside/reply")], policy="map-prefix")),
        ("user_properties_limite_com_marcador", lambda: rejected_message(fixture, "users-limit", [(0x26, ("tag", str(index))) for index in range(16)])),
        ("maximum_qos_recusado_sem_reducao", lambda: capacity_case(fixture, "max-qos", [(0x24, 0)])),
        ("retain_indisponivel_recusado", lambda: capacity_case(fixture, "no-retain", [(0x25, 0)], retain=True)),
        ("maximum_packet_size_recusado", lambda: capacity_case(fixture, "max-size", [(0x27, 128)], payload=b"x" * 256)),
        ("puback_negativo_sem_ack_origem", lambda: negative_ack(fixture, 0x40)),
        ("pubrec_negativo_sem_ack_origem", lambda: negative_ack(fixture, 0x50)),
        ("pubcomp_negativo_sem_ack_origem", lambda: negative_ack(fixture, 0x70)),
        ("expiry_replay_desconta_tempo", lambda: expiry_recovery(fixture)),
        ("sessao_destino_ausente_preserva_journal", lambda: missing_session(fixture, "destination")),
        ("sessao_origem_ausente_preserva_journal", lambda: missing_session(fixture, "source")),
        ("sessao_destino_desconhecida_recusada", lambda: unknown_session(fixture, "destination")),
        ("sessao_origem_desconhecida_recusada", lambda: unknown_session(fixture, "source")),
        ("qos2_duplicata_pendente_sem_nova_entrega", lambda: qos2_duplicates(fixture)),
        ("qos2_reuso_pid_apos_pubcomp_dup0_mesmo_payload", lambda: qos2_reuse(fixture, False, False)),
        ("qos2_reuso_pid_apos_pubcomp_dup0_outro_payload", lambda: qos2_reuse(fixture, False, True)),
        ("qos2_reuso_pid_apos_pubcomp_dup1_mesmo_payload", lambda: qos2_reuse(fixture, True, False)),
        ("qos2_reuso_pid_apos_pubcomp_dup1_outro_payload", lambda: qos2_reuse(fixture, True, True)),
        ("alias_subid_wire_somente_se_negociados", lambda: optional_connection_properties(fixture)),
    ])
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bridge", type=Path, required=True)
    parser.add_argument("--broker-root", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--only", action="append", default=[], help="Executar somente este cenário (repetível).")
    args = parser.parse_args()
    available = {name for name, _ in scenarios(None)}
    if set(args.only) - available:
        parser.error("--only deve indicar um nome de cenário existente")
    if os.name != "posix":
        parser.error("execute em Linux/WSL para validar modos privados e sinais")
    if args.report.exists() or args.report.is_symlink():
        parser.error("o relatório deve ser novo; preserve os FAIL anteriores")
    bridge, broker_root = args.bridge.resolve(), args.broker_root.resolve()
    if not bridge.is_file() or not os.access(bridge, os.X_OK):
        parser.error("--bridge deve indicar um executável existente")
    sys.path.insert(0, str(broker_root / "scripts"))
    from verify_wss_mtls import certificates
    sources = [Path(__file__).resolve(), *[Path(__file__).resolve().parents[1] / "src" / name for name in
                ["bridge_metadata.rs", "bridge_store.rs", "bridge_v5.rs", "bridge_multi.rs", "bin/mqtt-bridge.rs"]]]
    initial_sources = {str(path.relative_to(Path(__file__).resolve().parents[1])): digest(path) for path in sources}
    report = dict(prompt="C148-v1.14", scope="wire TLS/MQTT5 finito; não conclui fase8/interop/crash integral",
                  started_utc=datetime.now(timezone.utc).isoformat(), bridge_sha256=digest(bridge),
                  selection=args.only or "all",
                  source_sha256=initial_sources, scenarios=[])
    began = time.monotonic()
    try:
        with tempfile.TemporaryDirectory(prefix="xmqr-c148-wire-") as directory:
            root = Path(directory)
            root.chmod(0o700)
            certificates(root)
            for path in root.iterdir():
                if path.is_file():
                    path.chmod(0o600)
            fixture = Fixture(bridge, root)
            for name, action in scenarios(fixture):
                if args.only and name not in args.only:
                    continue
                try:
                    evidence = action() or {}
                    status = evidence.pop("status", "PASS")
                    row = dict(name=name, status=status, evidence=evidence)
                except Exception as error:
                    last = traceback.extract_tb(error.__traceback__)[-1]
                    row = dict(name=name, status="FAIL", error=f"{type(error).__name__}: {error}"[:700],
                               location=f"{Path(last.filename).name}:{last.lineno}:{last.name}")
                report["scenarios"].append(row)
                print(json.dumps(dict(name=row["name"], status=row["status"])), flush=True)
    except Exception as error:
        report["scenarios"].append(dict(name="preparacao_fixture", status="FAIL", error=f"{type(error).__name__}: {error}"[:700]))
    report["bridge_unchanged"] = digest(bridge) == report["bridge_sha256"]
    report["sources_unchanged"] = all(digest(path) == initial_sources[str(path.relative_to(Path(__file__).resolve().parents[1]))] for path in sources)
    report["counts"] = {status: sum(row["status"] == status for row in report["scenarios"])
                        for status in ["PASS", "FAIL", "NOT_RUN"]}
    report["status"] = "PASS" if not report["counts"]["FAIL"] and report["bridge_unchanged"] and report["sources_unchanged"] else "FAIL"
    report["elapsed_seconds"] = round(time.monotonic() - began, 3)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    with args.report.open("x", encoding="utf-8") as stream:
        json.dump(report, stream, indent=2, ensure_ascii=False)
        stream.write("\n")
    print(json.dumps(dict(status=report["status"], counts=report["counts"], report=str(args.report))), flush=True)
    return int(report["status"] != "PASS")


if __name__ == "__main__":
    raise SystemExit(main())
