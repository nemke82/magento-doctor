<?php
return [
    'backend' => [
        'frontName' => 'admin_secret'
    ],
    'crypt' => [
        'key' => 'mock_crypt_key_do_not_expose_12345'
    ],
    'db' => [
        'table_prefix' => '',
        'connection' => [
            'default' => [
                'host' => '127.0.0.1',
                'dbname' => 'magento_doctor_test',
                'username' => 'mdoctor_user',
                'password' => 'secret_mock_password_never_print',
                'model' => 'mysql4',
                'engine' => 'innodb',
                'initStatements' => 'SET NAMES utf8;',
                'active' => '1'
            ]
        ]
    ],
    'resource' => [
        'default_setup' => [
            'connection' => 'default'
        ]
    ],
    'x-frame-options' => 'SAMEORIGIN',
    'MAGE_MODE' => 'production',
    'session' => [
        'save' => 'redis',
        'redis' => [
            'host' => '127.0.0.1',
            'port' => '6379',
            'password' => 'redis_pass_secret',
            'database' => '2',
            'compression_threshold' => '2048',
            'compression_library' => 'gzip'
        ]
    ],
    'cache' => [
        'frontend' => [
            'default' => [
                'backend' => 'Magento\\Framework\\Cache\\Backend\\RemoteHash',
                'backend_options' => [
                    'server' => '127.0.0.1',
                    'port' => '6379',
                    'database' => '0'
                ]
            ],
            'page_cache' => [
                'backend' => 'Magento\\Framework\\Cache\\Backend\\RemoteHash',
                'backend_options' => [
                    'server' => '127.0.0.1',
                    'port' => '6379',
                    'database' => '1'
                ]
            ]
        ]
    ]
];
