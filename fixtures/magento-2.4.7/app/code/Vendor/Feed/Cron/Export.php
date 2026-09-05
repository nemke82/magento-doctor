<?php
namespace Vendor\Feed\Cron;

class Export {
    protected $productRepository;
    protected $logger;

    public function __construct(
        \Magento\Catalog\Api\ProductRepositoryInterface $productRepository,
        \Psr\Log\LoggerInterface $logger
    ) {
        $this->productRepository = $productRepository;
        $this->logger = $logger;
    }

    public function execute() {
        $products = []; // collection

        // N+1 entity access anti-pattern
        foreach ($products as $product) {
            $loaded = $this->productRepository->getById($product->getId());
            $this->logger->info("Loaded product " . $loaded->getSku());
        }

        // Outbound synchronous HTTP call
        $ch = curl_init("https://api.thirdparty-feed.com/v1/upload");
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
        $response = curl_exec($ch);
        curl_close($ch);

        return $response;
    }
}
