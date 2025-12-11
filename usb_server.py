# import serial
# import json
# import re
# from http.server import HTTPServer, BaseHTTPRequestHandler
# from threading import Thread
# import time

# latest_data = {
#     "latitude": 49.8180161,
#     "longitude": 24.022562,
#     "timestamp": None
# }

# class CoordinateHandler(BaseHTTPRequestHandler):
#     def do_GET(self):
#         if self.path == '/coordinates':
#             self.send_response(200)
#             self.send_header('Content-type', 'application/json')
#             self.send_header('Access-Control-Allow-Origin', '*')
#             self.end_headers()
#             self.wfile.write(json.dumps(latest_data).encode())
#         else:
#             self.send_response(404)
#             self.end_headers()
    
#     def log_message(self, format, *args):
#         pass

# def read_usb_serial():
#     global latest_data
#     buffer = ""

#     ports_to_try = [
#         '/dev/ttyACM0', '/dev/ttyACM1', 
#         '/dev/ttyUSB0', '/dev/ttyUSB1',
#         '/dev/cu.usbmodem*',
#         'COM3', 'COM4', 'COM5'
#     ]

#     ser = None
#     for port in ports_to_try:
#         try:
#             if '*' in port:
#                 import glob
#                 matching_ports = glob.glob(port)
#                 if matching_ports:
#                     port = matching_ports[0]
#                 else:
#                     continue
            
#             ser = serial.Serial(port, 115200, timeout=1)
#             print(f"Connected to {port}")
#             break
#         except:
#             continue

#     if not ser:
#         print("ERROR: Could not find USB serial device!")
#         print("Please check your Raspberry Pi Pico is connected")
#         return

#     print("Reading USB data... Server running on http://0.0.0.0:8080")

#     while True:
#         try:
#             if ser.in_waiting > 0:
#                 data = ser.read(ser.in_waiting).decode('utf-8', errors='ignore')
#                 buffer += data

#                 print(f'Data: {data}')
#                 latest_data["raw"] = data
#                 latest_data["timestamp"] = time.strftime('%H:%M:%S')

#                 # json_matches = re.findall(r'\{[^}]+\}', buffer)
                
#                 # for json_str in json_matches:
#                 #     try:
#                 #         parsed = json.loads(json_str)
#                 #         if 'lat' in parsed and 'long' in parsed:
#                 #             latest_data['latitude'] = parsed['lat'] / 10000000
#                 #             latest_data['longitude'] = parsed['long'] / 10000000
#                 #             latest_data['timestamp'] = time.strftime('%H:%M:%S')
#                 #             print(f"Updated: {latest_data['latitude']:.7f}, {latest_data['longitude']:.7f}")
#                 #     except json.JSONDecodeError:
#                 #         pass

#                 if len(buffer) > 1000:
#                     buffer = buffer[-500:]
            
#             time.sleep(0.1)
#         except Exception as e:
#             print(f"Error reading serial: {e}")
#             time.sleep(1)

# if __name__ == '__main__':
#     usb_thread = Thread(target=read_usb_serial, daemon=True)
#     usb_thread.start()
#     server = HTTPServer(('0.0.0.0', 8080), CoordinateHandler)
#     print("HTTP Server started on port 8080")
#     print("Access at: http://localhost:8080/coordinates")
#     server.serve_forever()

import json
import time
from http.server import HTTPServer, BaseHTTPRequestHandler

latest_data = {
    "latitude": 49.8174252,
    "longitude": 24.0246498,
    "timestamp": None
}

class CoordinateHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/coordinates':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps(latest_data).encode())
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        pass

def update_fake_data():
    global latest_data
    lat = 498174252
    lon = 240246498
    
    while True:
        lat += 1000
        lon += 500
        latest_data['latitude'] = lat / 10000000
        latest_data['longitude'] = lon / 10000000
        latest_data['timestamp'] = time.strftime('%H:%M:%S')
        print(f"Test data: {latest_data['latitude']:.7f}, {latest_data['longitude']:.7f}")
        time.sleep(3)

if __name__ == '__main__':
    from threading import Thread
    
    data_thread = Thread(target=update_fake_data, daemon=True)
    data_thread.start()
    
    server = HTTPServer(('0.0.0.0', 8080), CoordinateHandler)
    print("TEST Server started on port 8080")
    print("Generating fake coordinates...")
    server.serve_forever()