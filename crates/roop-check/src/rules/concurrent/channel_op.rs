/// One send or receive a task performs on a channel, in program order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelOp {
    pub chan: String,
    pub send: bool,
}
