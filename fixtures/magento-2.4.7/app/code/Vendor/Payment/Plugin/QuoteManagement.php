<?php
namespace Vendor\Payment\Plugin;

class QuoteManagement {
    protected $client;

    public function __construct(\GuzzleHttp\Client $client) {
        $this->client = $client;
    }

    public function aroundSubmit($subject, callable $proceed, ...$args) {
        // Synchronous outbound HTTP call on checkout hot path
        $response = $this->client->request('POST', 'https://gateway.payment.com/v1/auth');

        return $proceed(...$args);
    }
}
