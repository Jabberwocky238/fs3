use crate::types::s3::request::*;
use crate::types::s3::response::S3Response;
use crate::types::traits::s3_handler::ObjectS3Handler;

macro_rules! object_api {
    ($fn_name:ident, $req_ty:ty, $method:ident, $variant:ident) => {
        pub async fn $fn_name<T, E>(handler: &T, req: $req_ty) -> Result<S3Response, E>
        where
            T: ObjectS3Handler<Error = E> + Send + Sync,
            E: Send + Sync + 'static,
        {
            Ok(S3Response::$variant(handler.$method(req).await?))
        }
    };
}

object_api!(head_object, HeadObjectRequest, head_object, HeadObject);
object_api!(
    get_object_attributes,
    GetObjectAttributesRequest,
    get_object_attributes,
    GetObjectAttributes
);
object_api!(
    copy_object_part,
    CopyObjectPartRequest,
    copy_object_part,
    CopyObjectPart
);
object_api!(
    put_object_part,
    PutObjectPartRequest,
    put_object_part,
    PutObjectPart
);
object_api!(
    list_object_parts,
    ListObjectPartsRequest,
    list_object_parts,
    ListObjectParts
);
object_api!(
    complete_multipart_upload,
    CompleteMultipartUploadRequest,
    complete_multipart_upload,
    CompleteMultipartUpload
);
object_api!(
    new_multipart_upload,
    NewMultipartUploadRequest,
    new_multipart_upload,
    NewMultipartUpload
);
object_api!(
    abort_multipart_upload,
    AbortMultipartUploadRequest,
    abort_multipart_upload,
    AbortMultipartUpload
);
object_api!(get_object_acl, GetObjectAclRequest, get_object_acl, GetObjectAcl);
object_api!(put_object_acl, PutObjectAclRequest, put_object_acl, PutObjectAcl);
object_api!(
    get_object_tagging,
    GetObjectTaggingRequest,
    get_object_tagging,
    GetObjectTagging
);
object_api!(
    put_object_tagging,
    PutObjectTaggingRequest,
    put_object_tagging,
    PutObjectTagging
);
object_api!(
    delete_object_tagging,
    DeleteObjectTaggingRequest,
    delete_object_tagging,
    DeleteObjectTagging
);
object_api!(
    select_object_content,
    SelectObjectContentRequest,
    select_object_content,
    SelectObjectContent
);
object_api!(
    get_object_retention,
    GetObjectRetentionRequest,
    get_object_retention,
    GetObjectRetention
);
object_api!(
    get_object_legal_hold,
    GetObjectLegalHoldRequest,
    get_object_legal_hold,
    GetObjectLegalHold
);
object_api!(
    get_object_lambda,
    GetObjectLambdaRequest,
    get_object_lambda,
    GetObjectLambda
);
object_api!(get_object, GetObjectRequest, get_object, GetObject);
object_api!(copy_object, CopyObjectRequest, copy_object, CopyObject);
object_api!(
    put_object_retention,
    PutObjectRetentionRequest,
    put_object_retention,
    PutObjectRetention
);
object_api!(
    put_object_legal_hold,
    PutObjectLegalHoldRequest,
    put_object_legal_hold,
    PutObjectLegalHold
);
object_api!(
    put_object_extract,
    PutObjectExtractRequest,
    put_object_extract,
    PutObjectExtract
);
object_api!(
    append_object_rejected,
    AppendObjectRejectedRequest,
    append_object_rejected,
    AppendObjectRejected
);
object_api!(put_object, PutObjectRequest, put_object, PutObject);
object_api!(delete_object, DeleteObjectRequest, delete_object, DeleteObject);
object_api!(
    post_restore_object,
    PostRestoreObjectRequest,
    post_restore_object,
    PostRestoreObject
);
