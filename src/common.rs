use std::ops::Deref;

use ::errors::*;
use ::backend::{
    MidiInputPort as MidiInputPortImpl,
    MidiInput as MidiInputImpl, 
    MidiInputConnection as MidiInputConnectionImpl,
    MidiOutputPort as MidiOutputPortImpl,
    MidiOutput as MidiOutputImpl,
    MidiOutputConnection as MidiOutputConnectionImpl
};
use ::Ignore;

// TODO: documentation
pub struct MidiInputPort {
    pub(crate) imp: MidiInputPortImpl
}

// TODO: documentation
pub type MidiInputPorts = Vec<MidiInputPort>;

/// An instance of `MidiInput` is required for anything related to MIDI input.
/// Create one with `MidiInput::new`.
pub struct MidiInput {
    //ignore_flags: Ignore
    imp: MidiInputImpl
}

impl MidiInput {
    /// Creates a new `MidiInput` object that is required for any MIDI input functionality.
    pub fn new(client_name: &str) -> Result<Self, InitError> {
        MidiInputImpl::new(client_name).map(|imp| MidiInput { imp: imp })
    }
    
    /// Set flags to decide what kind of messages should be ignored (i.e., filtered out)
    /// by this `MidiInput`. By default, no messages are ignored.
    pub fn ignore(&mut self, flags: Ignore) {
       self.imp.ignore(flags);
    }

    // TODO: documentation
    pub fn ports(&self) -> MidiInputPorts {
        self.imp.ports_internal()
    }
    
    /// Get the number of available MIDI input ports that *midir* can connect to.
    pub fn port_count(&self) -> usize {
        self.imp.port_count()
    }
    
    /// Get the name of a specified MIDI input port.
    pub fn port_name(&self, port: &MidiInputPort) -> Result<String, PortInfoError> {
        self.imp.port_name(&port.imp)
    }
    
    /// Connect to a specified MIDI input port in order to receive messages.
    /// For each incoming MIDI message, the provided `callback` function will
    /// be called. The first parameter of the callback function is a timestamp
    /// (in microseconds) designating the time since some unspecified point in
    /// the past (which will not change during the lifetime of a
    /// `MidiInputConnection`). The second parameter contains the actual bytes
    /// of the MIDI message.
    ///
    /// Additional data that should be passed whenever the callback is
    /// invoked can be specified by `data`. Use the empty tuple `()` if
    /// you do not want to pass any additional data.
    ///
    /// The connection will be kept open as long as the returned
    /// `MidiInputConnection` is kept alive.
    ///
    /// The `port_name` is an additional name that will be assigned to the
    /// connection. It is only used by some backends.
    pub fn connect<F, T: Send>(
        self, port: &MidiInputPort, port_name: &str, callback: F, data: T
    ) -> Result<MidiInputConnection<T>, ConnectError<MidiInput>>
        where F: FnMut(u64, &[u8], &mut T) + Send + 'static {
        match self.imp.connect(&port.imp, port_name, callback, data) {
            Ok(imp) => Ok(MidiInputConnection { imp: imp }),
            Err(imp) => {
                let kind = imp.kind();
                Err(ConnectError::new(kind, MidiInput { imp: imp.into_inner() }))
            } 
        }
    }
}

#[cfg(unix)]
impl<T: Send> ::os::unix::VirtualInput<T> for MidiInput {
    fn create_virtual<F>(
        self, port_name: &str, callback: F, data: T
    ) -> Result<MidiInputConnection<T>, ConnectError<Self>>
    where F: FnMut(u64, &[u8], &mut T) + Send + 'static {
        match self.imp.create_virtual(port_name, callback, data) {
            Ok(imp) => Ok(MidiInputConnection { imp: imp }),
            Err(imp) => {
                let kind = imp.kind();
                Err(ConnectError::new(kind, MidiInput { imp: imp.into_inner() }))
            } 
        }
    }
}

/// Represents an open connection to a MIDI input port.
pub struct MidiInputConnection<T: 'static> {
    imp: MidiInputConnectionImpl<T>
}

impl<T> MidiInputConnection<T> {
    /// Closes the connection. The returned values allow you to
    /// inspect the additional data passed to the callback (the `data`
    /// parameter of `connect`), or to reuse the `MidiInput` object,
    /// but they can be safely ignored.
    pub fn close(self) -> (MidiInput, T) {
        let (imp, data) = self.imp.close();
        (MidiInput { imp: imp }, data)
    }
}

// TODO: documentation
pub struct MidiOutputPort {
    pub(crate) imp: MidiOutputPortImpl
}

// TODO: documentation
pub struct MidiOutputPorts {
    inner: Vec<MidiOutputPort>
}

impl Deref for MidiOutputPorts {
    type Target = [MidiOutputPort];

    fn deref(&self) -> &[MidiOutputPort] {
        &self.inner
    }
}

/// An instance of `MidiOutput` is required for anything related to MIDI output.
/// Create one with `MidiOutput::new`.
pub struct MidiOutput {
    imp: MidiOutputImpl
}

impl MidiOutput {
    /// Creates a new `MidiOutput` object that is required for any MIDI output functionality.
    pub fn new(client_name: &str) -> Result<Self, InitError> {
        MidiOutputImpl::new(client_name).map(|imp| MidiOutput { imp: imp })
    }

    // TODO: documentation
    pub fn ports(&self) -> MidiOutputPorts {
        MidiOutputPorts { inner: self.imp.ports_internal() }
    }
    
    /// Get the number of available MIDI output ports that *midir* can connect to.
    pub fn port_count(&self) -> usize {
        self.imp.port_count()
    }
    
    /// Get the name of a specified MIDI output port.
    pub fn port_name(&self, port: &MidiOutputPort) -> Result<String, PortInfoError> {
        self.imp.port_name(&port.imp)
    }
    
    /// Connect to a specified MIDI output port in order to send messages.
    /// The connection will be kept open as long as the returned
    /// `MidiOutputConnection` is kept alive.
    ///
    /// The `port_name` is an additional name that will be assigned to the
    /// connection. It is only used by some backends.
    pub fn connect(self, port: &MidiOutputPort, port_name: &str) -> Result<MidiOutputConnection, ConnectError<MidiOutput>> {
        match self.imp.connect(&port.imp, port_name) {
            Ok(imp) => Ok(MidiOutputConnection { imp: imp }),
            Err(imp) => {
                let kind = imp.kind();
                Err(ConnectError::new(kind, MidiOutput { imp: imp.into_inner() }))
            } 
        }
    }
}

#[cfg(unix)]
impl ::os::unix::VirtualOutput for MidiOutput {
    fn create_virtual(self, port_name: &str) -> Result<MidiOutputConnection, ConnectError<MidiOutput>> {
        match self.imp.create_virtual(port_name) {
            Ok(imp) => Ok(MidiOutputConnection { imp: imp }),
            Err(imp) => {
                let kind = imp.kind();
                Err(ConnectError::new(kind, MidiOutput { imp: imp.into_inner() }))
            } 
        }
    }
}

/// Represents an open connection to a MIDI output port.
pub struct MidiOutputConnection {
   imp: MidiOutputConnectionImpl
}

impl MidiOutputConnection {
    /// Closes the connection. The returned value allows you to
    /// reuse the `MidiOutput` object, but it can be safely ignored.
    pub fn close(self) -> MidiOutput {
        MidiOutput { imp: self.imp.close() }
    }
    
    /// Send a message to the port that this output connection is connected to.
    /// The message must be a valid MIDI message (see https://www.midi.org/specifications/item/table-1-summary-of-midi-message).
    pub fn send(&mut self, message: &[u8]) -> Result<(), SendError> {
        self.imp.send(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_impls() {
        // make sure that all the structs implement `Send`
        fn is_send<T: Send>() {}
        is_send::<MidiInputPort>();
        is_send::<MidiInput>();
        is_send::<MidiInputConnection<()>>();
        is_send::<MidiOutputPort>();
        is_send::<MidiOutput>();
        is_send::<MidiOutputConnection>();
    }
}
