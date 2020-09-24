use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    /*#[fail(display = "Error message: {}", message)]
    Message {
        message: String,
    },*/
    #[fail(display = "Transport operation failed: {}", message)]
    TransportError {
        message: String,
        #[fail(cause)] cause: tonic::transport::Error,
    },
    /*#[fail(display = "I/O operation failed: {}", message)]
    IOError {
        message: String,
        #[fail(cause)] cause: std::io::Error,
    },*/
}
