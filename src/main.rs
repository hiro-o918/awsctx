use crate::ctxm::view::show_contexts;
use crate::aws::AWS;
use crate::ctxm::CTXM;

mod aws;
mod ctxm;

fn main() {
    let cli = "aws";
    let command = "list";

    let aws = AWS::default();
    let contexts = aws.list_contexts().unwrap();
    show_contexts(&contexts);
}
