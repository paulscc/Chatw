# ProyectChat Frontend

Frontend básico para la aplicación de chat empresarial ProyectChat.

## Características

- Interfaz moderna con TailwindCSS
- Conexión WebSocket en tiempo real
- Diseño responsivo
- Chat por canales y mensajes directos

## Tecnologías

- HTML5
- TailwindCSS (CDN)
- JavaScript nativo
- WebSocket API

## Ejecución local

```bash
# Opción 1: Con Python
python -m http.server 3000

# Opción 2: Con Node.js (si lo prefieres)
npx serve . -p 3000
```

Luego abre `http://localhost:3000` en tu navegador.

## Configuración

El frontend se conecta al backend WebSocket en:
```
ws://localhost:8080/ws/user123
```

Para producción, actualiza la URL WebSocket en `index.html` con la URL de tu backend desplegado en Render.

## Despliegue en Render

1. Sube esta carpeta a un repositorio GitHub separado
2. Crea un nuevo servicio Web en Render
3. Render detectará automáticamente que es un sitio estático HTML

## Variables de entorno

No requiere variables de entorno para el frontend. La configuración del backend WebSocket se hace directamente en el código JavaScript.
