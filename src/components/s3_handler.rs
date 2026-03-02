
pub struct Handler<Engine, E: S3EngineError> {
    pub engine: Engine,
    pub _error: std::marker::PhantomData<E>,
}

impl<Engine, E: S3EngineError> Handler<Engine, E> {
    pub fn new(engine: Engine) -> Self {
        Self { engine, _error: std::marker::PhantomData }
    }
}

// Handler impls — engine error E is used directly as handler error
impl<Engine, E> RootS3Handler<E> for Handler<Engine, E>
where
    Engine: S3BucketEngine<E> + Send + Sync,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    fn engine(&self) -> &Engine { &self.engine }
}

impl<Engine, E> BucketS3Handler<E> for Handler<Engine, E>
where
    Engine: S3Engine<E> + Send + Sync,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    fn engine(&self) -> &Engine { &self.engine }
}

impl<Engine, E> ObjectS3Handler<E> for Handler<Engine, E>
where
    Engine: S3ObjectEngine<E> + S3MultipartEngine<E> + Send + Sync,
    E: S3EngineError + From<S3HandlerBridgeError>,
{
    type Engine = Engine;
    fn engine(&self) -> &Engine { &self.engine }
}

#[async_trait]
impl<Engine, E> RejectedS3Handler<E> for Handler<Engine, E>
where
    Engine: Send + Sync + 'static,
    E: S3EngineError,
{
}