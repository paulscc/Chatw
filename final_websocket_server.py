#!/usr/bin/env python3
"""
Servidor WebSocket final - sintaxis alternativa que funciona
"""

import asyncio
import json
import logging
from datetime import datetime

# Configurar logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Almacenamiento de conexiones por sala
rooms = {}

class SimpleWebSocketServer:
    def __init__(self):
        self.rooms = {}
    
    async def register_client(self, websocket, room_code):
        """Registrar cliente en una sala"""
        if room_code not in self.rooms:
            self.rooms[room_code] = set()
        self.rooms[room_code].add(websocket)
        logger.info(f"Cliente registrado en sala {room_code}. Total: {len(self.rooms[room_code])}")
        return room_code
    
    async def unregister_client(self, websocket, room_code):
        """Desregistrar cliente de una sala"""
        if room_code in self.rooms:
            self.rooms[room_code].discard(websocket)
            if not self.rooms[room_code]:
                del self.rooms[room_code]
                logger.info(f"Sala {room_code} eliminada")
    
    async def broadcast_to_room(self, room_code, message):
        """Enviar mensaje a todos los clientes de una sala"""
        if room_code in self.rooms:
            disconnected = []
            for client in self.rooms[room_code]:
                try:
                    await client.send(json.dumps(message))
                except Exception as e:
                    logger.error(f"Error enviando a cliente: {e}")
                    disconnected.append(client)
            
            # Limpiar clientes desconectados
            for client in disconnected:
                self.rooms[room_code].discard(client)

server = SimpleWebSocketServer()

async def websocket_handler(websocket):
    """Handler principal con la firma correcta"""
    try:
        logger.info(f"Nueva conexión WebSocket: {websocket.remote_address}")
        
        # Esperar mensaje inicial del cliente
        async for message in websocket:
            try:
                data = json.loads(message)
                
                # Procesar mensaje de unión a sala
                if data.get("type") == "join":
                    room_code = data.get("room_code", "default")
                    logger.info(f"Cliente uniéndose a sala: {room_code}")
                    
                    # Registrar cliente
                    await server.register_client(websocket, room_code)
                    
                    # Enviar mensaje de bienvenida
                    welcome = {
                        "type": "message",
                        "data": {
                            "content": f"¡Conectado a la sala {room_code}!",
                            "sender_id": "system",
                            "timestamp": datetime.utcnow().isoformat(),
                            "isOwn": False
                        }
                    }
                    await websocket.send(json.dumps(welcome))
                
                # Procesar mensaje normal
                elif data.get("type") == "message":
                    room_code = data.get("room_code", "default")
                    content = data.get("content", "")
                    
                    if content:
                        # Crear mensaje de broadcast
                        broadcast = {
                            "type": "message",
                            "data": {
                                "content": content,
                                "sender_id": data.get("sender", "unknown"),
                                "timestamp": datetime.utcnow().isoformat(),
                                "room_code": room_code
                            }
                        }
                        
                        # Enviar a todos en la sala
                        await server.broadcast_to_room(room_code, broadcast)
                        logger.info(f"Mensaje broadcast a sala {room_code}: {content}")
                
            except json.JSONDecodeError:
                logger.error("Mensaje JSON inválido")
            except Exception as e:
                logger.error(f"Error procesando mensaje: {e}")
    
    except Exception as e:
        logger.error(f"Error en handler: {e}")
    finally:
        # Cleanup
        try:
            # Aquí podríamos hacer cleanup si fuera necesario
            pass
        except:
            pass

async def main():
    """Iniciar servidor"""
    host = "127.0.0.1"
    port = 8087
    
    logger.info(f"Iniciando servidor WebSocket en {host}:{port}")
    logger.info("Endpoint: ws://127.0.0.1:8087")
    
    # Usar sintaxis alternativa para websockets
    import websockets
    
    server_instance = await websockets.serve(websocket_handler, host, port)
    logger.info(f"Servidor iniciado en ws://{host}:{port}")
    
    await server_instance.wait_closed()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        logger.info("Servidor detenido")
    except Exception as e:
        logger.error(f"Error: {e}")
