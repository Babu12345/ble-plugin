import tkinter as tk
from tkinter import ttk, scrolledtext, messagebox
from plugin_host.comms import USBHostDevice, USBCommunicationError, validate_mac_address
import threading
import time
import usb.core
import plugin_host.protocol_pb2 as protocol_pb2

class BLEConfigurationGUI:
    def __init__(self, root):
        self.root = root
        self.root.title("BLE USB Configuration Tool")
        self.root.geometry("900x700")
        
        self.host = None
        self.is_connected = False
        self.device_available = False
        self.connection_monitor_thread = None
        self.monitor_running = False
        
        
        # Message listening variables
        self.message_listener_thread = None
        self.is_listening = False
        self.message_count = 0
        self.listener_paused = False  # Flag to pause listener for manual operations
        self.message_window = None  # Separate window for messages
        self.show_raw_data = False  # Toggle for raw vs deserialized view
        
        self.setup_ui()
        self.start_connection_monitor()
        self.check_device_availability()  # Initial check
        
    def setup_ui(self):
        # Create a frame with scrollbar for the notebook
        notebook_container = ttk.Frame(self.root)
        notebook_container.pack(fill="both", expand=True, padx=10, pady=10)
        
        # Create canvas and scrollbar for horizontal scrolling
        notebook_canvas = tk.Canvas(notebook_container, highlightthickness=0, borderwidth=0)
        notebook_scrollbar = ttk.Scrollbar(notebook_container, orient="horizontal", command=notebook_canvas.xview)
        notebook_canvas.configure(xscrollcommand=notebook_scrollbar.set)
        
        # Pack scrollbar at bottom, canvas fills rest
        notebook_scrollbar.pack(side="bottom", fill="x")
        notebook_canvas.pack(side="top", fill="both", expand=True)
        
        # Create notebook inside canvas with minimum width to ensure all tabs are visible
        notebook = ttk.Notebook(notebook_canvas)
        
        # Create window in canvas
        canvas_window = notebook_canvas.create_window((0, 0), window=notebook, anchor="nw")
        
        # Function to handle canvas and notebook sizing
        def configure_notebook_canvas(event=None):
            # Get canvas dimensions
            canvas_width = notebook_canvas.winfo_width()
            canvas_height = notebook_canvas.winfo_height()
            
            # Calculate minimum width needed for all tabs (estimate)
            min_width = 1000  # Minimum width to show all tabs without compression
            
            # Use the larger of canvas width or minimum width
            notebook_width = max(canvas_width, min_width)
            
            # Configure the notebook size
            notebook_canvas.itemconfig(canvas_window, width=notebook_width, height=canvas_height)
            
            # Update scroll region to enable scrolling if needed
            notebook_canvas.configure(scrollregion=(0, 0, notebook_width, canvas_height))
        
        # Bind resize to canvas configure event
        notebook_canvas.bind("<Configure>", configure_notebook_canvas)
        
        self.connection_frame = ttk.Frame(notebook)
        self.peripheral_frame = ttk.Frame(notebook)
        self.service_frame = ttk.Frame(notebook)
        self.characteristic_frame = ttk.Frame(notebook)
        self.profile_frame = ttk.Frame(notebook)
        self.messages_frame = ttk.Frame(notebook)
        self.log_frame = ttk.Frame(notebook)
        
        notebook.add(self.connection_frame, text="Connection")
        notebook.add(self.peripheral_frame, text="Peripheral")
        notebook.add(self.service_frame, text="Services")
        notebook.add(self.characteristic_frame, text="Characteristics")
        notebook.add(self.profile_frame, text="Profile & Advertisement")
        notebook.add(self.messages_frame, text="Incoming Messages")
        notebook.add(self.log_frame, text="Logs")
        
        self.setup_connection_tab()
        self.setup_peripheral_tab()
        self.setup_service_tab()
        self.setup_characteristic_tab()
        self.setup_profile_tab()
        self.setup_messages_tab()
        self.setup_log_tab()
        
        status_frame = ttk.Frame(self.root)
        status_frame.pack(fill="x", padx=10, pady=5)
        
        # Add visual indicator canvas
        self.indicator_canvas = tk.Canvas(status_frame, width=30, height=30, highlightthickness=0)
        self.indicator_canvas.pack(side="left", padx=5)
        self.create_status_indicator()
        
        # Status label with indicator
        self.status_label = ttk.Label(status_frame, text="Status: Disconnected", foreground="red")
        self.status_label.pack(side="left")
        
        # Add auto-detection indicator
        self.auto_detect_label = ttk.Label(status_frame, text="  [Auto-detection: Active]", foreground="gray")
        self.auto_detect_label.pack(side="left")
        
        # Add last check time
        self.last_check_label = ttk.Label(status_frame, text="", foreground="gray")
        self.last_check_label.pack(side="right")
        
        # Animation variables
        self.animation_id = None
        self.pulse_size = 0
        self.pulse_direction = 1
        
    def setup_connection_tab(self):
        frame = ttk.LabelFrame(self.connection_frame, text="USB Connection", padding=10)
        frame.pack(fill="both", expand=True, padx=10, pady=10)
        
        ttk.Label(frame, text="Command Delay (seconds):").grid(row=0, column=0, sticky="w", pady=5)
        self.delay_var = tk.StringVar(value="0.1")
        ttk.Entry(frame, textvariable=self.delay_var, width=20).grid(row=0, column=1, pady=5)
        
        ttk.Label(frame, text="Connection Sleep Time (seconds):").grid(row=1, column=0, sticky="w", pady=5)
        self.sleep_var = tk.StringVar(value="0.5")
        ttk.Entry(frame, textvariable=self.sleep_var, width=20).grid(row=1, column=1, pady=5)
        
        button_frame = ttk.Frame(frame)
        button_frame.grid(row=3, column=0, columnspan=2, pady=20)
        
        self.connect_btn = ttk.Button(button_frame, text="Connect", command=self.connect_device)
        self.connect_btn.pack(side="left", padx=5)
        
        self.disconnect_btn = ttk.Button(button_frame, text="Disconnect", command=self.disconnect_device, state="disabled")
        self.disconnect_btn.pack(side="left", padx=5)
        
    def setup_peripheral_tab(self):
        frame = ttk.LabelFrame(self.peripheral_frame, text="Peripheral Configuration", padding=10)
        frame.pack(fill="both", expand=True, padx=10, pady=10)
        
        ttk.Label(frame, text="Peripheral Name:").grid(row=0, column=0, sticky="w", pady=5)
        self.peripheral_name_var = tk.StringVar(value="Example Peripheral")
        ttk.Entry(frame, textvariable=self.peripheral_name_var, width=30).grid(row=0, column=1, pady=5)
        
        ttk.Label(frame, text="MAC Address (6 hex bytes):").grid(row=1, column=0, sticky="w", pady=5)
        mac_frame = ttk.Frame(frame)
        mac_frame.grid(row=1, column=1, pady=5)
        
        self.mac_vars = []
        for i in range(6):
            var = tk.StringVar(value=f"{0x01}")
            self.mac_vars.append(var)
            entry = ttk.Entry(mac_frame, textvariable=var, width=4)
            entry.pack(side="left", padx=2)
            if i < 5:
                ttk.Label(mac_frame, text=":").pack(side="left")
        
        ttk.Button(frame, text="Configure Peripheral", command=self.configure_peripheral).grid(row=2, column=0, columnspan=2, pady=10)
        
        security_frame = ttk.LabelFrame(frame, text="Security", padding=10)
        security_frame.grid(row=3, column=0, columnspan=2, sticky="ew", pady=10)
        
        ttk.Label(security_frame, text="Passkey:").grid(row=0, column=0, sticky="w", pady=5)
        self.passkey_var = tk.StringVar(value="123456")
        ttk.Entry(security_frame, textvariable=self.passkey_var, width=20).grid(row=0, column=1, pady=5)
        
        ttk.Button(security_frame, text="Configure Security", command=self.configure_security).grid(row=1, column=0, columnspan=2, pady=5)
        
    def setup_service_tab(self):
        frame = ttk.LabelFrame(self.service_frame, text="Service Configuration", padding=10)
        frame.pack(fill="both", expand=True, padx=10, pady=10)
        
        ttk.Label(frame, text="Service UUID (16-bit hex):").grid(row=0, column=0, sticky="w", pady=5)
        self.service_uuid_var = tk.StringVar(value="0x8765")
        ttk.Entry(frame, textvariable=self.service_uuid_var, width=20).grid(row=0, column=1, pady=5)
        
        ttk.Button(frame, text="Configure Service", command=self.configure_service).grid(row=1, column=0, columnspan=2, pady=10)
        
        query_frame = ttk.LabelFrame(frame, text="Query Service", padding=10)
        query_frame.grid(row=2, column=0, columnspan=2, sticky="ew", pady=10)
        
        ttk.Label(query_frame, text="Query UUID:").grid(row=0, column=0, sticky="w", pady=5)
        self.query_uuid_var = tk.StringVar(value="0x8765")
        ttk.Entry(query_frame, textvariable=self.query_uuid_var, width=20).grid(row=0, column=1, pady=5)
        
        ttk.Button(query_frame, text="Query Service Info", command=self.query_service).grid(row=1, column=0, columnspan=2, pady=5)
        
        self.service_info_text = tk.Text(query_frame, height=5, width=50)
        self.service_info_text.grid(row=2, column=0, columnspan=2, pady=5)
        
    def setup_characteristic_tab(self):
        frame = ttk.LabelFrame(self.characteristic_frame, text="Characteristic Configuration", padding=10)
        frame.pack(fill="both", expand=True, padx=10, pady=10)
        
        ttk.Label(frame, text="Characteristic UUID (16-bit hex):").grid(row=0, column=0, sticky="w", pady=5)
        self.char_uuid_var = tk.StringVar(value="0xabcd")
        ttk.Entry(frame, textvariable=self.char_uuid_var, width=20).grid(row=0, column=1, pady=5)
        
        ttk.Label(frame, text="Service UUID (16-bit hex):").grid(row=1, column=0, sticky="w", pady=5)
        self.char_service_uuid_var = tk.StringVar(value="0x8765")
        ttk.Entry(frame, textvariable=self.char_service_uuid_var, width=20).grid(row=1, column=1, pady=5)
        
        ttk.Label(frame, text="Properties:").grid(row=2, column=0, sticky="nw", pady=5)
        
        props_frame = ttk.Frame(frame)
        props_frame.grid(row=2, column=1, pady=5)
        
        self.prop_vars = {}
        properties = ["READ", "WRITE", "NOTIFY", "INDICATE", "WRITE_WITHOUT_RESPONSE"]
        for i, prop in enumerate(properties):
            var = tk.BooleanVar(value=(prop in ["READ", "WRITE", "NOTIFY"]))
            self.prop_vars[prop] = var
            ttk.Checkbutton(props_frame, text=prop, variable=var).grid(row=i//2, column=i%2, sticky="w")
        
        ttk.Button(frame, text="Configure Characteristic", command=self.configure_characteristic).grid(row=3, column=0, columnspan=2, pady=10)
        
    def setup_profile_tab(self):
        frame = ttk.LabelFrame(self.profile_frame, text="Profile & Advertisement", padding=10)
        frame.pack(fill="both", expand=True, padx=10, pady=10)
        
        ttk.Label(frame, text="Profile:").grid(row=0, column=0, sticky="w", pady=5)
        self.profile_var = tk.StringVar(value="CUSTOM")
        profile_combo = ttk.Combobox(frame, textvariable=self.profile_var, width=20)
        profile_combo['values'] = ["CUSTOM", "HeartRate", "Cycling", "Running"]
        profile_combo.grid(row=0, column=1, pady=5)
        
        ttk.Label(frame, text="Profile Delay (seconds):").grid(row=1, column=0, sticky="w", pady=5)
        self.profile_delay_var = tk.StringVar(value="0.05")
        ttk.Entry(frame, textvariable=self.profile_delay_var, width=20).grid(row=1, column=1, pady=5)
        
        ttk.Button(frame, text="Configure Profile", command=self.configure_profile).grid(row=2, column=0, columnspan=2, pady=10)
        
        adv_frame = ttk.LabelFrame(frame, text="Advertisement", padding=10)
        adv_frame.grid(row=3, column=0, columnspan=2, sticky="ew", pady=10)
        
        self.multi_connect_var = tk.BooleanVar(value=True)
        ttk.Checkbutton(adv_frame, text="Allow Multiple Connections", variable=self.multi_connect_var).grid(row=0, column=0, pady=5)
        
        ttk.Button(adv_frame, text="Start Advertisement", command=self.start_advertisement).grid(row=1, column=0, pady=5)
        ttk.Button(adv_frame, text="Stop Advertisement", command=self.stop_advertisement).grid(row=1, column=1, pady=5)
    
    def setup_messages_tab(self):
        frame = ttk.Frame(self.messages_frame)
        frame.pack(fill="both", expand=True, padx=10, pady=10)
        
        # Control buttons at the top
        control_frame = ttk.LabelFrame(frame, text="Message Listening Controls", padding=10)
        control_frame.pack(fill="x", pady=(0, 10))
        
        # Create inner frame for controls
        controls_inner = ttk.Frame(control_frame)
        controls_inner.pack(fill="x")
        
        # Add buttons directly to frame
        self.start_listening_btn = ttk.Button(controls_inner, text="Start Listening", command=self.start_listening)
        self.start_listening_btn.pack(side="left", padx=5)
        
        self.stop_listening_btn = ttk.Button(controls_inner, text="Stop Listening", command=self.stop_listening, state="disabled")
        self.stop_listening_btn.pack(side="left", padx=5)
        
        ttk.Button(controls_inner, text="Clear Messages", command=self.clear_messages).pack(side="left", padx=5)
        
        # Toggle for raw/deserialized view
        self.raw_data_var = tk.BooleanVar(value=self.show_raw_data)
        ttk.Checkbutton(controls_inner, text="Show Raw Data", variable=self.raw_data_var, 
                       command=self.toggle_raw_data).pack(side="left", padx=10)
        
        # Button to open separate window
        ttk.Button(controls_inner, text="Open in Separate Window", command=self.open_message_window).pack(side="left", padx=10)
        
        # Status indicator
        self.listening_status_label = ttk.Label(controls_inner, text="Status: Not Listening", foreground="red")
        self.listening_status_label.pack(side="right", padx=10)
        
        # Message count
        self.message_count_label = ttk.Label(controls_inner, text="Messages: 0", foreground="gray")
        self.message_count_label.pack(side="right", padx=10)
        
        # Messages display area
        message_frame = ttk.LabelFrame(frame, text="Incoming Messages", padding=10)
        message_frame.pack(fill="both", expand=True)
        
        # Create scrolled text for messages
        self.messages_text = scrolledtext.ScrolledText(message_frame, height=20, width=80, wrap=tk.WORD)
        self.messages_text.pack(fill="both", expand=True)
        
        # Configure text tags for different message types
        self.messages_text.tag_configure("timestamp", foreground="gray", font=("Courier", 9))
        self.messages_text.tag_configure("data", foreground="blue", font=("Courier", 9))
        self.messages_text.tag_configure("error", foreground="red", font=("Courier", 9))
        self.messages_text.tag_configure("response", foreground="green", font=("Courier", 9))
        
    def setup_log_tab(self):
        frame = ttk.Frame(self.log_frame)
        frame.pack(fill="both", expand=True, padx=10, pady=10)
        
        self.log_text = scrolledtext.ScrolledText(frame, height=20, width=80)
        self.log_text.pack(fill="both", expand=True)
        
        button_frame = ttk.Frame(frame)
        button_frame.pack(fill="x", pady=5)
        
        ttk.Button(button_frame, text="Clear Logs", command=self.clear_logs).pack(side="right")
        
    def log(self, message, level="INFO"):
        timestamp = time.strftime("%H:%M:%S")
        self.log_text.insert(tk.END, f"[{timestamp}] [{level}] {message}\n")
        self.log_text.see(tk.END)
    
        
    def connect_device(self):
        try:
            delay = float(self.delay_var.get())
            sleep_time = float(self.sleep_var.get())
            
            self.host = USBHostDevice(default_command_delay=delay)
            self.log("Connecting to USB device...")
            
            if self.host.connect(sleep_time=sleep_time):
                self.is_connected = True
                self.device_available = True
                self.status_label.config(text="Status: Connected", foreground="green")
                self.update_status_indicator("connected")
                self.connect_btn.config(state="disabled")
                self.disconnect_btn.config(state="normal")
                self.last_check_label.config(text="")
                self.log("✓ Connected successfully", "SUCCESS")
                self.log("✓ Auto-detection enabled - monitoring USB connection", "INFO")
            else:
                self.host = None
                self.log("✗ Failed to connect", "ERROR")
                messagebox.showerror("Connection Failed", "Failed to connect to USB device. Please ensure the device is connected.")
                
        except USBCommunicationError as e:
            self.host = None
            self.log(f"USB Communication error: {e}", "ERROR")
            messagebox.showerror("USB Error", f"Could not communicate with USB device:\n{e}")
        except Exception as e:
            self.host = None
            self.log(f"Connection error: {e}", "ERROR")
            messagebox.showerror("Connection Error", str(e))
            
    def disconnect_device(self):
        if self.host:
            # Stop listening if active
            if self.is_listening:
                self.stop_listening()
                
            self.host.disconnect()
            self.is_connected = False
            self.host = None
            self.status_label.config(text="Status: Disconnected", foreground="red")
            self.update_status_indicator("disconnected")
            self.connect_btn.config(state="normal")
            self.disconnect_btn.config(state="disabled")
            self.last_check_label.config(text="")
            self.log("✓ Disconnected", "SUCCESS")
            
    def configure_peripheral(self):
        if not self.is_connected:
            messagebox.showwarning("Not Connected", "Please connect to device first")
            return
            
        try:
            name = self.peripheral_name_var.get()
            mac_bytes = []
            for var in self.mac_vars:
                mac_bytes.append(int(var.get(), 16))
            
            # Validate MAC address size
            validation = validate_mac_address(mac_bytes)
            if validation is not None:
                messagebox.showerror("Invalid MAC Address", validation)
                return
                
            self.log(f"Configuring peripheral: {name}")
            
            # Pause listener for manual operation
            with self.pause_listener():
                self.host.configure_peripheral(name=name, addr=mac_bytes)
                
            self.log("✓ Peripheral configured successfully", "SUCCESS")
            
        except Exception as e:
            self.log(f"Failed to configure peripheral: {e}", "ERROR")
            messagebox.showerror("Configuration Error", str(e))
            
    def configure_security(self):
        if not self.is_connected:
            messagebox.showwarning("Not Connected", "Please connect to device first")
            return
            
        try:
            passkey = int(self.passkey_var.get())
            self.log(f"Configuring security with passkey: {passkey}")
            
            with self.pause_listener():
                self.host.configure_peripheral_security(passkey=passkey)
                
            self.log("✓ Security configured successfully", "SUCCESS")
            
        except Exception as e:
            self.log(f"Failed to configure security: {e}", "ERROR")
            messagebox.showerror("Security Error", str(e))
            
    def configure_service(self):
        if not self.is_connected:
            messagebox.showwarning("Not Connected", "Please connect to device first")
            return
            
        try:
            uuid_str = self.service_uuid_var.get()
            uuid = int(uuid_str, 16) if uuid_str.startswith("0x") else int(uuid_str)
            
            self.log(f"Configuring service: {uuid_str}")
            
            with self.pause_listener():
                self.host.configure_service(uuid=uuid)
                
            self.log("✓ Service configured successfully", "SUCCESS")
            
        except Exception as e:
            self.log(f"Failed to configure service: {e}", "ERROR")
            messagebox.showerror("Service Error", str(e))
            
    def query_service(self):
        if not self.is_connected:
            messagebox.showwarning("Not Connected", "Please connect to device first")
            return
            
        try:
            uuid_str = self.query_uuid_var.get()
            uuid = int(uuid_str, 16) if uuid_str.startswith("0x") else int(uuid_str)
            
            self.log(f"Querying service: {uuid_str}")
            
            # Pause listener to give priority to manual operation
            with self.pause_listener():
                service_info = self.host.get_service_info(uuid)
            
            info_text = f"Service exists: {service_info.exists}\n"
            info_text += f"Characteristics: {len(service_info.characteristic_uuids)}\n"
            if service_info.characteristic_uuids:
                info_text += "UUIDs: " + ", ".join([f"0x{u:04x}" for u in service_info.characteristic_uuids])
                
            self.service_info_text.delete(1.0, tk.END)
            self.service_info_text.insert(1.0, info_text)
            self.log("✓ Service queried successfully", "SUCCESS")
            
        except USBCommunicationError as e:
            self.log(f"Service query failed: {e}", "WARNING")
            self.service_info_text.delete(1.0, tk.END)
            self.service_info_text.insert(1.0, f"Query failed: {e}")
            
    def configure_characteristic(self):
        if not self.is_connected:
            messagebox.showwarning("Not Connected", "Please connect to device first")
            return
            
        try:
            char_uuid_str = self.char_uuid_var.get()
            char_uuid = int(char_uuid_str, 16) if char_uuid_str.startswith("0x") else int(char_uuid_str)
            
            service_uuid_str = self.char_service_uuid_var.get()
            service_uuid = int(service_uuid_str, 16) if service_uuid_str.startswith("0x") else int(service_uuid_str)
            
            properties = []
            for prop_name, var in self.prop_vars.items():
                if var.get():
                    properties.append(getattr(protocol_pb2.BLEProperties, prop_name))
                    
            self.log(f"Configuring characteristic: {char_uuid_str} for service {service_uuid_str}")
            
            with self.pause_listener():
                self.host.configure_characteristic(
                    uuid=char_uuid,
                    service_uuid=service_uuid,
                    properties=properties
                )
                
            self.log("✓ Characteristic configured successfully", "SUCCESS")
            
        except Exception as e:
            self.log(f"Failed to configure characteristic: {e}", "ERROR")
            messagebox.showerror("Characteristic Error", str(e))
            
    def configure_profile(self):
        if not self.is_connected:
            messagebox.showwarning("Not Connected", "Please connect to device first")
            return
            
        try:
            profile_name = self.profile_var.get()
            profile = getattr(protocol_pb2.BleProfile, profile_name)
            delay = float(self.profile_delay_var.get())
            
            self.log(f"Configuring profile: {profile_name} with delay {delay}s")
            
            with self.pause_listener():
                self.host.configure_profile(profile, delay=delay)
                
            self.log("✓ Profile configured successfully", "SUCCESS")
            
        except Exception as e:
            self.log(f"Failed to configure profile: {e}", "ERROR")
            messagebox.showerror("Profile Error", str(e))
            
    def start_advertisement(self):
        if not self.is_connected:
            messagebox.showwarning("Not Connected", "Please connect to device first")
            return
            
        try:
            allow_multi = self.multi_connect_var.get()
            self.log(f"Starting advertisement (multi-connect: {allow_multi})")
            
            with self.pause_listener():
                self.host.start_advertisement(allow_multi_connect=allow_multi)
                
            self.log("✓ Advertisement started successfully", "SUCCESS")
            
        except Exception as e:
            self.log(f"Failed to start advertisement: {e}", "ERROR")
            messagebox.showerror("Advertisement Error", str(e))
            
    def stop_advertisement(self):
        if not self.is_connected:
            messagebox.showwarning("Not Connected", "Please connect to device first")
            return
            
        try:
            self.log("Stopping advertisement")
            
            with self.pause_listener():
                self.host.stop_advertisement()
                
            self.log("✓ Advertisement stopped successfully", "SUCCESS")
            
        except Exception as e:
            self.log(f"Failed to stop advertisement: {e}", "ERROR")
            messagebox.showerror("Advertisement Error", str(e))
            
    def clear_logs(self):
        self.log_text.delete(1.0, tk.END)
    
    def pause_listener(self):
        """Context manager to pause the message listener during manual operations"""
        class ListenerPauser:
            def __init__(self, gui):
                self.gui = gui
                
            def __enter__(self):
                self.gui.listener_paused = True
                # Update UI to show listener is paused
                if self.gui.is_listening:
                    self.gui.root.after(0, lambda: self.gui.listening_status_label.config(
                        text="Status: Paused (Manual Operation)", foreground="orange"
                    ))
                    # Update separate window status if it exists
                    if self.gui.message_window and self.gui.message_window.winfo_exists():
                        self.gui.root.after(0, lambda: self.gui.message_window_status_label.config(
                            text="Status: Paused (Manual Operation)", foreground="orange"
                        ))
                # Wait a bit to ensure listener has paused
                time.sleep(0.15)  # Give listener time to finish current operation
                return self
                
            def __exit__(self, exc_type, exc_val, exc_tb):
                self.gui.listener_paused = False
                # Update UI to show listener is resumed
                if self.gui.is_listening:
                    self.gui.root.after(0, lambda: self.gui.listening_status_label.config(
                        text="Status: Listening", foreground="green"
                    ))
                    # Update separate window status if it exists
                    if self.gui.message_window and self.gui.message_window.winfo_exists():
                        self.gui.root.after(0, lambda: self.gui.message_window_status_label.config(
                            text="Status: Listening", foreground="green"
                        ))
                return False
        
        return ListenerPauser(self)
    
    def start_listening(self):
        """Start listening for incoming messages"""
        if not self.is_connected:
            messagebox.showwarning("Not Connected", "Please connect to device first")
            return
            
        if self.is_listening:
            return
            
        self.is_listening = True
        self.message_count = 0
        self.listening_status_label.config(text="Status: Listening", foreground="green")
        self.start_listening_btn.config(state="disabled")
        self.stop_listening_btn.config(state="normal")
        
        # Update separate window controls if it exists
        if self.message_window and self.message_window.winfo_exists():
            self.message_window_start_btn.config(state="disabled")
            self.message_window_stop_btn.config(state="normal")
            self.message_window_status_label.config(text="Status: Listening", foreground="green")
        
        # Start the message listener thread
        self.message_listener_thread = threading.Thread(target=self.message_listener, daemon=True)
        self.message_listener_thread.start()
        
        self.log("Started listening for incoming messages", "INFO")
        self.add_message("=== Message listening started ===", "info")
    
    def stop_listening(self):
        """Stop listening for incoming messages"""
        self.is_listening = False
        self.listening_status_label.config(text="Status: Not Listening", foreground="red")
        self.start_listening_btn.config(state="normal")
        self.stop_listening_btn.config(state="disabled")
        
        # Update separate window controls if it exists
        if self.message_window and self.message_window.winfo_exists():
            self.message_window_start_btn.config(state="normal")
            self.message_window_stop_btn.config(state="disabled")
            self.message_window_status_label.config(text="Status: Not Listening", foreground="red")
        
        self.log("Stopped listening for incoming messages", "INFO")
        self.add_message("=== Message listening stopped ===", "info")
    
    def clear_messages(self):
        """Clear the messages display"""
        self.messages_text.delete(1.0, tk.END)
        if self.message_window and self.message_window.winfo_exists():
            self.message_window_text.delete(1.0, tk.END)
        self.message_count = 0
        self.message_count_label.config(text="Messages: 0")
        if self.message_window and self.message_window.winfo_exists():
            self.message_window_count_label.config(text="Messages: 0")
        self.log("Messages cleared", "INFO")
    
    def open_message_window(self):
        """Open or focus the separate message window"""
        if self.message_window and self.message_window.winfo_exists():
            # Window already exists, bring it to front
            self.message_window.lift()
            self.message_window.focus()
        else:
            # Create new window
            self.create_message_window()
    
    def create_message_window(self):
        """Create a separate window for messages"""
        self.message_window = tk.Toplevel(self.root)
        self.message_window.title("Incoming Messages - BLE USB Monitor")
        self.message_window.geometry("600x700")
        
        # Position to the right of main window
        main_x = self.root.winfo_x()
        main_y = self.root.winfo_y()
        main_width = self.root.winfo_width()
        self.message_window.geometry(f"+{main_x + main_width + 10}+{main_y}")
        
        # Control frame at top
        control_frame = ttk.Frame(self.message_window)
        control_frame.pack(fill="x", padx=10, pady=10)
        
        # Control buttons
        self.message_window_start_btn = ttk.Button(control_frame, text="Start Listening", 
                                                   command=self.start_listening)
        self.message_window_start_btn.pack(side="left", padx=5)
        
        self.message_window_stop_btn = ttk.Button(control_frame, text="Stop Listening", 
                                                  command=self.stop_listening, state="disabled")
        self.message_window_stop_btn.pack(side="left", padx=5)
        
        ttk.Button(control_frame, text="Clear", command=self.clear_messages).pack(side="left", padx=5)
        
        # Toggle for raw/deserialized view in separate window
        self.window_raw_data_var = tk.BooleanVar(value=self.show_raw_data)
        ttk.Checkbutton(control_frame, text="Show Raw Data", variable=self.window_raw_data_var,
                       command=self.toggle_window_raw_data).pack(side="left", padx=5)
        
        # Auto-scroll checkbox
        self.auto_scroll_var = tk.BooleanVar(value=True)
        ttk.Checkbutton(control_frame, text="Auto-scroll", 
                       variable=self.auto_scroll_var).pack(side="left", padx=10)
        
        # Status indicators
        status_frame = ttk.Frame(self.message_window)
        status_frame.pack(fill="x", padx=10, pady=5)
        
        self.message_window_status_label = ttk.Label(status_frame, 
                                                     text="Status: Not Listening", foreground="red")
        self.message_window_status_label.pack(side="left", padx=5)
        
        self.message_window_count_label = ttk.Label(status_frame, 
                                                    text="Messages: 0", foreground="gray")
        self.message_window_count_label.pack(side="right", padx=5)
        
        # Messages display
        message_frame = ttk.LabelFrame(self.message_window, text="Incoming Messages", padding=10)
        message_frame.pack(fill="both", expand=True, padx=10, pady=(0, 10))
        
        # Scrolled text widget
        self.message_window_text = scrolledtext.ScrolledText(message_frame, height=30, width=70, wrap=tk.WORD)
        self.message_window_text.pack(fill="both", expand=True)
        
        # Configure text tags
        self.message_window_text.tag_configure("timestamp", foreground="gray", font=("Courier", 9))
        self.message_window_text.tag_configure("data", foreground="blue", font=("Courier", 10, "bold"))
        self.message_window_text.tag_configure("error", foreground="red", font=("Courier", 10))
        self.message_window_text.tag_configure("response", foreground="green", font=("Courier", 10, "bold"))
        self.message_window_text.tag_configure("info", foreground="gray", font=("Courier", 9, "italic"))
        
        # Copy existing messages if any
        if self.messages_text.get(1.0, tk.END).strip():
            self.message_window_text.insert(1.0, self.messages_text.get(1.0, tk.END))
        
        # Update button states to match current listening state
        if self.is_listening:
            self.message_window_start_btn.config(state="disabled")
            self.message_window_stop_btn.config(state="normal")
            self.message_window_status_label.config(text="Status: Listening", foreground="green")
        
        # Update count
        self.message_window_count_label.config(text=f"Messages: {self.message_count}")
        
        # Handle window close
        self.message_window.protocol("WM_DELETE_WINDOW", self.on_message_window_close)
    
    def on_message_window_close(self):
        """Handle message window close event"""
        if self.message_window:
            self.message_window.destroy()
            self.message_window = None
    
    def add_message(self, message, msg_type="data"):
        """Add a message to the messages display with timestamp and formatting"""
        timestamp = time.strftime("%H:%M:%S.") + f"{int(time.time() * 1000) % 1000:03d}"
        
        # Insert to main window
        self.messages_text.insert(tk.END, f"[{timestamp}] ", "timestamp")
        
        # Insert message with appropriate tag
        if msg_type == "error":
            self.messages_text.insert(tk.END, f"ERROR: {message}\n", "error")
        elif msg_type == "response":
            self.messages_text.insert(tk.END, f"RESPONSE: {message}\n", "response")
        elif msg_type == "info":
            self.messages_text.insert(tk.END, f"{message}\n", "timestamp")
        else:
            self.messages_text.insert(tk.END, f"DATA: {message}\n", "data")
        
        # Auto-scroll to bottom
        self.messages_text.see(tk.END)
        
        # Also add to separate window if it exists
        if self.message_window and self.message_window.winfo_exists():
            self.message_window_text.insert(tk.END, f"[{timestamp}] ", "timestamp")
            
            if msg_type == "error":
                self.message_window_text.insert(tk.END, f"ERROR: {message}\n", "error")
            elif msg_type == "response":
                self.message_window_text.insert(tk.END, f"RESPONSE: {message}\n", "response")
            elif msg_type == "info":
                self.message_window_text.insert(tk.END, f"{message}\n", "info")
            else:
                self.message_window_text.insert(tk.END, f"DATA: {message}\n", "data")
            
            # Auto-scroll if enabled
            if hasattr(self, 'auto_scroll_var') and self.auto_scroll_var.get():
                self.message_window_text.see(tk.END)
        
        # Update message count if it's a real message (not info)
        if msg_type != "info":
            self.message_count += 1
            self.root.after(0, lambda: self.message_count_label.config(text=f"Messages: {self.message_count}"))
            if self.message_window and self.message_window.winfo_exists():
                self.root.after(0, lambda: self.message_window_count_label.config(text=f"Messages: {self.message_count}"))
    
    def format_response_message(self, raw_data):
        """Format raw USB response data as either raw bytes or deserialized based on toggle"""
        if self.show_raw_data:
            # Show raw hex data
            hex_data = ' '.join([f'{b:02X}' for b in raw_data])
            return f"Raw Bytes: {hex_data}"
        else:
            # Try to deserialize and show formatted data
            try:
                from plugin_host.comms import deserialize_response
                deserialized = deserialize_response(raw_data)
                
                if isinstance(deserialized, protocol_pb2.PluginData):
                    # Format PluginData fields prettily
                    src_addr = ':'.join([f'{b:02X}' for b in deserialized.src_addr])
                    send_type = str(deserialized.send_type).split('.')[-1]
                    char_uuid = f"0x{deserialized.characteristic_uuid:04X}"
                    service_uuid = f"0x{deserialized.service_uuid:04X}"
                    
                    # Format data payload with better readability
                    if deserialized.data:
                        data_hex = ' '.join([f'{b:02X}' for b in deserialized.data])
                        if len(deserialized.data) <= 16:
                            data_display = data_hex
                        else:
                            # Show first 16 bytes + count for longer data
                            first_16 = ' '.join([f'{b:02X}' for b in deserialized.data[:16]])
                            data_display = f"{first_16}... ({len(deserialized.data)} bytes total)"
                    else:
                        data_display = "(empty)"
                    
                    return (
                        f"📱 BLE Data Message:\n"
                        f"   Device: {src_addr} ({deserialized.src_addr_type})\n"
                        f"   Action: {send_type}\n"
                        f"   Service: {service_uuid}\n"
                        f"   Characteristic: {char_uuid}\n"
                        f"   Payload: {data_display}"
                    )
                else:
                    # For other message types, show basic info
                    return f"📦 {type(deserialized).__name__}"
                    
            except Exception as e:
                return f"⚠️  Unable to deserialize: {e}"
    
    def toggle_raw_data(self):
        """Toggle between raw and deserialized data view"""
        self.show_raw_data = self.raw_data_var.get()
        # Sync with separate window if it exists
        if hasattr(self, 'window_raw_data_var'):
            self.window_raw_data_var.set(self.show_raw_data)
    
    def toggle_window_raw_data(self):
        """Toggle raw data view from separate window"""
        self.show_raw_data = self.window_raw_data_var.get()
        # Sync with main window
        if hasattr(self, 'raw_data_var'):
            self.raw_data_var.set(self.show_raw_data)
    
    def message_listener(self):
        """Background thread that listens for incoming messages"""
        while self.is_listening:
            # Check if listener is paused for manual operations
            if self.listener_paused:
                time.sleep(0.05)  # Short sleep while paused
                continue
                
            if not self.host or not self.is_connected:
                time.sleep(0.1)
                continue
                
            try:
                # Try to receive a message from the USB device
                # The USBDevice class now handles thread safety internally
                response = self.host.usb_device.receive_data(timeout=100)  # 100ms timeout
                
                if response:
                    # Format and display the message
                    formatted_message = self.format_response_message(response)
                    self.root.after(0, lambda data=formatted_message: self.add_message(data, "response"))
                    
            except Exception as e:
                error_str = str(e).lower()
                # Silently ignore timeout errors - they're expected when no data
                if "timeout" in error_str or "timed out" in error_str or "operation timed out" in error_str:
                    pass  # Normal - no data available
                # Only show real errors
                elif "errno" in error_str or "disconnected" in error_str:
                    error_msg = f"Receive error: {e}"
                    self.root.after(0, lambda msg=error_msg: self.add_message(msg, "error"))
                
                # Small delay to prevent busy waiting
                time.sleep(0.05)
    
    def create_status_indicator(self):
        """Create the initial status indicator (USB icon-like shape)"""
        # Draw a simple USB connector shape
        self.indicator_base = self.indicator_canvas.create_rectangle(
            10, 12, 20, 22, fill="gray", outline="darkgray", width=2
        )
        # USB connector tip
        self.indicator_tip = self.indicator_canvas.create_rectangle(
            20, 14, 25, 20, fill="gray", outline="darkgray", width=1
        )
        # Connection status dot
        self.status_dot = self.indicator_canvas.create_oval(
            5, 5, 15, 15, fill="red", outline=""
        )
    
    def update_status_indicator(self, status):
        """Update the visual indicator based on connection status"""
        if status == "connected":
            # Green for connected
            self.indicator_canvas.itemconfig(self.indicator_base, fill="#4CAF50", outline="#2E7D32")
            self.indicator_canvas.itemconfig(self.indicator_tip, fill="#4CAF50", outline="#2E7D32")
            self.indicator_canvas.itemconfig(self.status_dot, fill="#4CAF50")
            self.stop_pulse_animation()
        elif status == "available":
            # Orange for detected but not connected - with pulsing animation
            self.indicator_canvas.itemconfig(self.indicator_base, fill="#FF9800", outline="#F57C00")
            self.indicator_canvas.itemconfig(self.indicator_tip, fill="#FF9800", outline="#F57C00")
            self.indicator_canvas.itemconfig(self.status_dot, fill="#FF9800")
            self.start_pulse_animation()
        else:
            # Gray/red for disconnected
            self.indicator_canvas.itemconfig(self.indicator_base, fill="gray", outline="darkgray")
            self.indicator_canvas.itemconfig(self.indicator_tip, fill="gray", outline="darkgray")
            self.indicator_canvas.itemconfig(self.status_dot, fill="red")
            self.stop_pulse_animation()
    
    def start_pulse_animation(self):
        """Start pulsing animation for 'available' state"""
        if self.animation_id is None:
            self.animate_pulse()
    
    def stop_pulse_animation(self):
        """Stop the pulsing animation"""
        if self.animation_id is not None:
            self.root.after_cancel(self.animation_id)
            self.animation_id = None
            # Reset the dot to normal size
            self.indicator_canvas.coords(self.status_dot, 5, 5, 15, 15)
    
    def animate_pulse(self):
        """Animate a pulsing effect on the status dot"""
        if not self.device_available or self.is_connected:
            self.animation_id = None
            return
            
        # Update pulse size
        self.pulse_size += self.pulse_direction * 0.5
        if self.pulse_size >= 3 or self.pulse_size <= 0:
            self.pulse_direction *= -1
        
        # Apply pulse to status dot
        center_x, center_y = 10, 10
        size = 5 + self.pulse_size
        self.indicator_canvas.coords(
            self.status_dot,
            center_x - size, center_y - size,
            center_x + size, center_y + size
        )
        
        # Continue animation
        self.animation_id = self.root.after(50, self.animate_pulse)
    
    def start_connection_monitor(self):
        """Start a background thread to monitor USB connection status"""
        if not self.monitor_running:
            self.monitor_running = True
            self.connection_monitor_thread = threading.Thread(target=self.monitor_connection, daemon=True)
            self.connection_monitor_thread.start()
    
    def check_device_availability(self):
        """Check if a USB device is available (plugged in) but not necessarily connected"""
        try:
            # Default USB IDs from the comms module
            vendor_id = 0xffff
            product_id = 0xffff
            
            device = usb.core.find(idVendor=vendor_id, idProduct=product_id)
            
            if device is not None:
                if not self.device_available:
                    self.device_available = True
                    if not self.is_connected:
                        self.root.after(0, lambda: self.update_device_status("available"))
                return True
            else:
                if self.device_available:
                    self.device_available = False
                    if not self.is_connected:
                        self.root.after(0, lambda: self.update_device_status("not_found"))
                return False
        except Exception as e:
            print(f"Error checking device availability: {e}")
            return False
    
    def update_device_status(self, status):
        """Update the status display based on device availability"""
        if status == "available":
            self.status_label.config(text="Status: Device Detected (Not Connected)", foreground="orange")
            self.update_status_indicator("available")
            self.log("USB device detected but not connected. Click Connect to establish connection.", "INFO")
        elif status == "not_found":
            self.status_label.config(text="Status: No Device Found", foreground="red")
            self.update_status_indicator("disconnected")
            self.log("No USB device found. Please plug in the device.", "WARNING")
    
    def monitor_connection(self):
        """Background thread that checks connection status every 2 seconds"""
        while self.monitor_running:
            # Always check device availability when not connected
            if not self.is_connected:
                self.check_device_availability()
            
            # Check connected device status
            if self.is_connected and self.host:
                try:
                    # Check if the underlying USB device still exists
                    # We check the usb_device.device attribute which is the actual pyusb device
                    if not self.host.usb_device or not self.host.usb_device.device:
                        # Device disconnected
                        self.root.after(0, self.handle_disconnection)
                        continue
                    
                    # Try to verify the device is still accessible
                    # PyUSB device objects become invalid when disconnected
                    try:
                        # Attempt to read the device's configuration (lightweight check)
                        config = self.host.usb_device.device.get_active_configuration()
                        if config is None:
                            self.root.after(0, self.handle_disconnection)
                            continue
                            
                        # Also try to check if we can still find the device
                        found_device = usb.core.find(
                            idVendor=self.host.usb_device.vendor_id,
                            idProduct=self.host.usb_device.product_id
                        )
                        if found_device is None:
                            self.root.after(0, self.handle_disconnection)
                            continue
                            
                        # Update last check time only if all checks pass
                        check_time = time.strftime("%H:%M:%S")
                        self.root.after(0, lambda ct=check_time: self.last_check_label.config(text=f"Last checked: {ct}"))
                        
                    except (usb.core.USBError, usb.core.USBTimeoutError) as e:
                        # USB errors indicate disconnection
                        print(f"USB Error detected: {e}")
                        self.root.after(0, self.handle_disconnection)
                    except (AttributeError, ValueError) as e:
                        # Device was invalidated
                        print(f"Device invalidated: {e}")
                        self.root.after(0, self.handle_disconnection)
                        
                except Exception as e:
                    # Any unexpected error likely means disconnection
                    print(f"Unexpected error in monitor: {e}")
                    self.root.after(0, self.handle_disconnection)
            
            time.sleep(2)  # Check every 2 seconds
    
    def handle_disconnection(self):
        """Handle USB disconnection detected by monitor"""
        if self.is_connected:
            self.log("⚠ USB device disconnected unexpectedly!", "WARNING")
            self.is_connected = False
            self.device_available = False
            self.host = None
            self.status_label.config(text="Status: Disconnected (USB removed)", foreground="red")
            self.update_status_indicator("disconnected")
            self.connect_btn.config(state="normal")
            self.disconnect_btn.config(state="disabled")
            
            # Show alert to user
            messagebox.showwarning(
                "USB Disconnected", 
                "The USB device has been disconnected. Please reconnect the device and click Connect."
            )
    
    def check_connection_health(self):
        """Perform a health check on the connection"""
        if self.host and self.is_connected:
            try:
                # Attempt a benign operation to check if device is responsive
                # This is a lightweight check that won't interfere with normal operations
                return self.host.device and self.host.device.is_open()
            except:
                return False
        return False
    
    def on_closing(self):
        self.monitor_running = False
        self.stop_pulse_animation()
        
        # Stop message listening
        if self.is_listening:
            self.stop_listening()
            
        if self.connection_monitor_thread:
            self.connection_monitor_thread.join(timeout=2)
        if self.host and self.is_connected:
            self.disconnect_device()
        self.root.destroy()


def main():
    root = tk.Tk()
    app = BLEConfigurationGUI(root)
    root.protocol("WM_DELETE_WINDOW", app.on_closing)
    root.mainloop()


if __name__ == "__main__":
    main()