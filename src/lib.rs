use std::{
    fs::File,
    io,
    io::Read,
    mem::{size_of, transmute},
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU16, Ordering},
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc,
    },
    thread,
    time::Duration,
};

use libc::input_event;

struct RawData
{
    rx: Receiver<u16>,
}

fn get_input_event(file: &mut File) -> Result<input_event, io::Error>
{
    let mut buf = [0; size_of::<input_event>()];
    file.read_exact(&mut buf)?;

    // SAFETY: Safe, because the dst type is ffi.
    Ok(unsafe { transmute(buf) })
}

fn generate_data<P: AsRef<Path>>(tx: Sender<u16>, path: P) -> Result<(), io::Error>
{
    let mut file = File::open(path)?;

    let mut x = 0;
    let mut y = 0;
    let val = Arc::new(AtomicU16::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let val_copy = val.clone();
    let stop_copy = stop.clone();

    thread::spawn(move || {
        let tx = tx;
        let val = val_copy;
        let stop = stop_copy;
        loop
        {
            if tx.send(val.load(Ordering::SeqCst)).is_err()
            {
                stop.store(true, Ordering::SeqCst);
            }
            thread::sleep(Duration::from_millis(8))
        }
    });

    loop
    {
        let event = get_input_event(&mut file)?;

        // Sync packet
        if event.type_ == 0
        {
            val.store((x * 256 + y) as _, Ordering::SeqCst);
        }
        // Data packet
        else if event.type_ == 3
        {
            if event.code == 0
            {
                x = event.value;
            }
            else if event.code == 1
            {
                y = event.value;
            }
        }

        if stop.load(Ordering::SeqCst)
        {
            // Other end was dropped, probably because the user want's
            // to use a other device or end the program.
            return Ok(());
        }
    }
}

impl RawData
{
    fn new<P>(path: P, tx_error: Sender<io::Error>) -> Self
    where
        P: AsRef<Path> + Send + 'static,
    {
        let (tx, rx) = channel();

        thread::spawn(move || match generate_data(tx, path)
        {
            Ok(()) =>
            {}
            Err(e) =>
            {
                let _ = tx_error.send(e);
            }
        });

        Self { rx }
    }
}

pub struct Data
{
    raw: RawData,
    data: Vec<u16>,
    rx_error: Receiver<io::Error>,
}

impl Data
{
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path> + Send + 'static,
    {
        let (tx_error, rx_error) = channel();

        Self {
            raw: RawData::new(path, tx_error),
            data: vec![],
            rx_error,
        }
    }

    pub fn poll(&mut self) -> bool
    {
        loop
        {
            match self.raw.rx.try_recv()
            {
                Ok(val) => self.data.push(val),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) =>
                {
                    if let Ok(err) = self.rx_error.recv()
                    {
                        eprintln!("Error in reading data: {:?}", err);
                    }
                    else
                    {
                        eprintln!("Error in reading data and in retrieving said error");
                    }

                    return false;
                }
            }
        }

        true
    }

    pub fn values(&self) -> &[u16]
    {
        &self.data
    }
}
