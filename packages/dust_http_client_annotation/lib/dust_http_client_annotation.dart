/// Support for core HttpClient annotations.
library;

export 'dart:convert';
export 'package:dio/dio.dart'
    show
        CancelToken,
        Dio,
        FormData,
        MultipartFile,
        Options,
        ProgressCallback,
        RequestOptions,
        Response,
        ResponseBody,
        ResponseType;
export 'src/http_client_annotations.dart';
