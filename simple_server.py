#!/usr/bin/env python3
"""
Servidor WebSocket temporal para pruebas mientras se resuelven problemas de compilación del backend Rust
"""

import asyncio
import websockets
import json
import logging
from datetime import datetime

# Configurar logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Almacenamiento de conexiones por sala
rooms = {}

async def handle_websocket(websocket, path):
    """
    Manejar conexiones WebSocket
    Path format: /ws/{room_code}
    """
    try:
        # Extraer room_code del path
        parts = path.strip('/').split('/')
        if len(parts) < 2 or parts[0] != 'ws':
            await websocket.close(1002, "Invalid path")
            return
        
        room_code = parts[1]
        logger.info(f"Nueva conexión WebSocket para sala: {room_code}")
        
        # Agregar conexión a la sala
        if room_code not in rooms:
            rooms[room_code] = set()
        rooms[room_code].add(websocket)
        
        # Enviar mensaje de bienvenida
        welcome_msg = {
            "type": "message",
            "data": {
                "content": f"Conectado a la sala {room_code}",
                "sender_id": "system",
                "timestamp": datetime.utcnow().isoformat(),
                "isOwn": False
            }
        }
        await websocket.send(json.dumps(welcome_msg))
        
        # Mantener conexión abierta y manejar mensajes
        async for message in websocket:
            try:
                data = json.loads(message)
                logger.info(f"Mensaje recibido en sala {room_code}: {data}")
                
                # Procesar diferentes tipos de mensajes
                if data.get("type") == "message" and data.get("data", {}).get("content"):
                    # Broadcast a todos los clientes en la sala
                    broadcast_msg = {
                        "type": "message",
                        "data": {
                            "content": data["data"]["content"],
                            "sender_id": data["data"].get("sender", "unknown"),
                            "timestamp": datetime.utcnow().isoformat(),
                            "room_code": room_code
                        }
                    }
                    
                    # Enviar a todos los clientes en la sala (incluyendo el remitente)
                    disconnected = set()
                    for client in rooms[room_code]:
                        try:
                            await client.send(json.dumps(broadcast_msg))
                        except websockets.exceptions.ConnectionClosed:
                            disconnected.add(client)
                        except Exception as e:
                            logger.error(f"Error enviando mensaje: {e}")
                            disconnected.add(client)
                    
                    # Limpiar conexiones desconectadas
                    for client in disconnected:
                        rooms[room_code].discard(client)
                
            except json.JSONDecodeError:
                logger.error(f"Mensaje JSON inválido: {message}")
            except Exception as e:
                logger.error(f"Error procesando mensaje: {e}")
    
    except websockets.exceptions.ConnectionClosed:
        logger.info(f"Conexión cerrada para sala: {room_code}")
    except Exception as e:
        logger.error(f"Error en conexión WebSocket: {e}")
    finally:
        # Limpiar conexión
        if room_code in rooms:
            rooms[room_code].discard(websocket)
            if not rooms[room_code]:
                del rooms[room_code]

async def main():
    """Iniciar servidor WebSocket"""
    host = "127.0.0.1"
    port = 8084
    
    logger.info(f"Iniciando servidor WebSocket en ws://{host}:{port}")
    logger.info("Endpoint: ws://127.0.0.1:8084/ws/{room_code}")
    
    async with websockets.serve(handle_websocket, host, port):
        logger.info("Servidor WebSocket iniciado correctamente")
        await asyncio.Future()  # Mantener servidor corriendo

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        logger.info("Servidor detenido")
    except Exception as e:
        logger.error(f"Error iniciando servidor: {e}")
