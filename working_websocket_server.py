#!/usr/bin/env python3
"""
Servidor WebSocket que funciona - versión final
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

class WebSocketServer:
    def __init__(self):
        self.rooms = {}
    
    async def handle_client(self, websocket, path):
        """Manejar cliente WebSocket"""
        try:
            logger.info(f"Cliente conectado: {websocket.remote_address}")
            logger.info(f"Path solicitado: {path}")
            
            # Extraer room_code del path
            if not path or not path.startswith('/ws/'):
                await websocket.close(1002, "Path inválido")
                return
            
            room_code = path.split('/')[-1]
            logger.info(f"Sala detectada: {room_code}")
            
            # Agregar a la sala
            if room_code not in self.rooms:
                self.rooms[room_code] = set()
            self.rooms[room_code].add(websocket)
            logger.info(f"Cliente agregado a sala {room_code}. Total en sala: {len(self.rooms[room_code])}")
            
            # Enviar mensaje de bienvenida
            welcome = {
                "type": "message",
                "data": {
                    "content": f"¡Bienvenido a la sala {room_code}!",
                    "sender_id": "system",
                    "timestamp": datetime.utcnow().isoformat(),
                    "isOwn": False
                }
            }
            await websocket.send(json.dumps(welcome))
            
            # Escuchar mensajes
            async for message in websocket:
                try:
                    data = json.loads(message)
                    logger.info(f"Mensaje recibido: {data}")
                    
                    if data.get("type") == "message":
                        content = data.get("data", {}).get("content", "")
                        if content:
                            # Crear mensaje de broadcast
                            broadcast = {
                                "type": "message",
                                "data": {
                                    "content": content,
                                    "sender_id": data.get("data", {}).get("sender", "unknown"),
                                    "timestamp": datetime.utcnow().isoformat(),
                                    "room_code": room_code
                                }
                            }
                            
                            # Enviar a todos en la sala
                            disconnected = []
                            for client in self.rooms[room_code]:
                                try:
                                    await client.send(json.dumps(broadcast))
                                    logger.info(f"Mensaje enviado a cliente")
                                except Exception as e:
                                    logger.error(f"Error enviando a cliente: {e}")
                                    disconnected.append(client)
                            
                            # Limpiar desconectados
                            for client in disconnected:
                                self.rooms[room_code].discard(client)
                
                except json.JSONDecodeError:
                    logger.error("Mensaje JSON inválido")
                except Exception as e:
                    logger.error(f"Error procesando mensaje: {e}")
        
        except Exception as e:
            logger.error(f"Error en conexión: {e}")
        finally:
            # Limpiar conexión
            try:
                if room_code in self.rooms:
                    self.rooms[room_code].discard(websocket)
                    logger.info(f"Cliente removido de sala {room_code}")
                    if not self.rooms[room_code]:
                        del self.rooms[room_code]
                        logger.info(f"Sala {room_code} eliminada")
            except:
                pass

async def main():
    """Iniciar servidor"""
    server = WebSocketServer()
    host = "127.0.0.1"
    port = 8086
    
    logger.info(f"Iniciando servidor WebSocket en {host}:{port}")
    logger.info("Endpoint: ws://127.0.0.1:8086/ws/{room_code}")
    
    # Usar websockets.serve con la sintaxis correcta
    import websockets
    
    async def handler(websocket, path):
        await server.handle_client(websocket, path)
    
    async with websockets.serve(handler, host, port):
        logger.info(f"Servidor iniciado en ws://{host}:{port}")
        await asyncio.Future()  # Mantener corriendo

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        logger.info("Servidor detenido")
    except Exception as e:
        logger.error(f"Error: {e}")
